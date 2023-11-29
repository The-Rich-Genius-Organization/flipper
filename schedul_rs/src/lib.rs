#[macro_use]
extern crate nickel;
extern crate lazy_static;

pub mod utils {
    use lazy_static::lazy_static;
    use std::time::Duration;
    use tokio;
    use tokio_cron_scheduler::{Job, JobScheduler, JobSchedulerError};
    use tokio_postgres::{Error, NoTls};

    pub const KAFKA_TOPIC: &str = "job_replica";
    pub const LOCAL_KAFKA: &str = "localhost:29092";
    pub const DOCKER_KAFKA: &str = "job_replica_queue:9092";
    pub const KAFKA_BROKER: &str = DOCKER_KAFKA;

    pub fn tok_rt() -> &'static tokio::runtime::Runtime {
        lazy_static! {
            static ref RT: tokio::runtime::Runtime =
                tokio::runtime::Runtime::new().expect("Should create a tokio runtime");
        }
        &RT
    }

    pub fn v1(path: &str) -> String {
        format!("/api/v1/{}", path)
    }

    pub async fn init_job_store(job_id: &str, seconds: u64) -> Result<(), Error> {
        let (client, conn) = tokio_postgres::connect(
            "host=db user=postgres password=example dbname=postgres",
            NoTls,
        )
        .await?;

        tokio::spawn(async move {
            if let Err(e) = conn.await {
                eprintln!("connection error: {}", e);
            }
        });

        client
            .execute(
                "INSERT INTO jobs (job_id, seconds) VALUES ($1, $2)",
                &[&job_id, &seconds.to_string()],
            )
            .await?;

        Ok(())
    }

    pub async fn take_job_store(job_id: &str) -> Result<i8, Error> {
        let (client, conn) = tokio_postgres::connect(
            "host=db user=postgres password=example dbname=postgres",
            NoTls,
        )
        .await?;

        tokio::spawn(async move {
            if let Err(e) = conn.await {
                eprintln!("connection error: {}", e);
            }
        });

        match client
            .query("SELECT taken FROM jobs WHERE job_id = $1", &[&job_id])
            .await?
            .first()
            .map(|row| row.get(0))
        {
            Some(true) => return Ok(1),
            _ => (),
        };

        client
            .execute("UPDATE jobs SET taken = TRUE WHERE job_id = $1", &[&job_id])
            .await?;

        Ok(0)
    }

    pub struct ShedRSScheduler {
        pub engine: JobScheduler,
    }

    impl ShedRSScheduler {
        pub async fn add_one_shot(
            &self,
            job_id: &str,
            seconds: u64,
        ) -> Result<(), JobSchedulerError> {
            let jid = String::from(job_id);
            let _ = init_job_store(&jid, seconds).await;

            self.engine
                .add(Job::new_one_shot(
                    Duration::from_secs(seconds),
                    move |_uuid, _l| {
                        let jid = jid.clone();
                        tok_rt().spawn(async move {
                            let res = take_job_store(&jid).await;
                            match res {
                                Ok(0) => println!("job {} underway", jid),
                                Ok(1) => println!("job {} already taken", jid),
                                _ => println!("job {} error", jid),
                            }
                        });
                    },
                )?)
                .await?;
            Ok(())
        }
    }

    pub async fn setup_db() -> Result<(), Error> {
        let (client, conn) = tokio_postgres::connect(
            "host=db user=postgres password=example dbname=postgres",
            NoTls,
        )
        .await?;

        tokio::spawn(async move {
            if let Err(e) = conn.await {
                eprintln!("connection error: {}", e);
            }
        });

        client
            .batch_execute(
                "
                CREATE TABLE IF NOT EXISTS jobs (
                    job_id          VARCHAR NOT NULL PRIMARY KEY,
                    seconds         VARCHAR NOT NULL,
                    taken           BOOLEAN NOT NULL DEFAULT FALSE
                )
            ",
            )
            .await?;

        Ok(())
    }
}

pub mod leader {
    use crate::utils;
    use kafka::producer::{Producer, Record, RequiredAcks};
    use nickel::{JsonBody, Nickel};
    use std::time::Duration;
    use tokio_cron_scheduler::JobScheduler;

    pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
        let mut server = Nickel::new();
        let sched = utils::ShedRSScheduler {
            engine: JobScheduler::new().await.unwrap(),
        };
        sched.engine.start().await.unwrap();

        loop {
            match utils::setup_db().await {
                Ok(_) => {
                    println!("DB Setup ... OK");
                    break;
                }
                Err(msg) => println!("ERROR {}", msg),
            }
            std::thread::sleep(Duration::from_secs(3));
        }

        fn send_to_kafka(topic: &str, value: &str) -> Result<(), kafka::Error> {
            let mut producer = Producer::from_hosts(vec![utils::KAFKA_BROKER.to_owned()])
                .with_ack_timeout(Duration::from_secs(1))
                .with_required_acks(RequiredAcks::One)
                .create()
                .unwrap();

            let record = Record::from_value(topic, value.as_bytes());
            let res = producer.send(&record)?;
            Ok(res)
        }

        server.utilize(router! {
            get utils::v1("health") => |_req, _res| {
                r#"{ "status" : "ok" }"#
            }

            post utils::v1("job") => |req, _res| {
                let data = req.json_as::<serde_json::Value>().unwrap();
                let job_id = data["job_id"].as_str().unwrap_or("null");
                let seconds = data["seconds"].as_u64().unwrap_or(0);

                let _ = utils::tok_rt().block_on(sched.add_one_shot(job_id, seconds));
                println!(
                    "job -> {}",
                    data.to_string()
                );

                let res = send_to_kafka(utils::KAFKA_TOPIC, &data.to_string());
                match res {
                    Ok(_) => r#"{ "status" : "ok" }"#,
                    _ => r#"{ "status" : "error" }"#,
                }
            }
        });

        server.listen("0.0.0.0:6767").unwrap();
        Ok(())
    }
}

pub mod follower {
    use crate::utils;
    use kafka::consumer::{Consumer, FetchOffset, GroupOffsetStorage};
    use serde_json::json;
    use std::thread::sleep;
    use std::time::Duration;
    use tokio_cron_scheduler::JobScheduler;

    fn connect_to_kafka() -> Result<Consumer, kafka::Error> {
        let consumer = Consumer::from_hosts(vec![utils::KAFKA_BROKER.to_owned()])
            .with_topic(utils::KAFKA_TOPIC.to_owned())
            .with_fallback_offset(FetchOffset::Earliest)
            .with_group("job_replica_group".to_owned())
            .with_offset_storage(Some(GroupOffsetStorage::Kafka))
            .create()?;
        Ok(consumer)
    }

    pub async fn run(delay: u64) -> Result<(), Box<dyn std::error::Error>> {
        let sched = utils::ShedRSScheduler {
            engine: JobScheduler::new().await.unwrap(),
        };
        sched.engine.start().await.unwrap();
        let mut consumer: Option<Consumer> = None;

        print!("Starting consumer ... ");
        while consumer.is_none() {
            consumer = match connect_to_kafka() {
                Ok(c) => {
                    println!("OK");
                    Some(c)
                }
                _ => None,
            };
            sleep(Duration::from_secs(3));
        }

        loop {
            match utils::setup_db().await {
                Ok(_) => {
                    print!("DB Setup ... OK");
                    break;
                }
                Err(msg) => println!("ERROR {}", msg),
            }
            std::thread::sleep(Duration::from_secs(3));
        }

        let c = consumer.as_mut().unwrap();
        loop {
            let mss = c.poll().unwrap();
            for ms in mss.iter() {
                for m in ms.messages() {
                    let data: serde_json::Value = serde_json::from_slice(m.value).unwrap();
                    let job_id = data["job_id"].as_str().unwrap_or("null");
                    let seconds = data["seconds"].as_u64().unwrap_or(0) + delay;

                    sched.add_one_shot(job_id, seconds).await.unwrap();
                    println!(
                        "job <- {}",
                        json!({ "job_id": job_id, "seconds": seconds }).to_string()
                    );
                    // println!(
                    //     "{}:{}@{}: {:?}",
                    //     ms.topic(),
                    //     ms.partition(),
                    //     m.offset,
                    //     m.value
                    // );
                }
                c.consume_messageset(ms).unwrap();
            }
            c.commit_consumed().unwrap();
        }
    }
}
