pub mod utils {
    use kafka::{
        client::KafkaClient,
        producer::{Producer, Record, RequiredAcks},
    };
    use lazy_static::lazy_static;
    use serde_json::json;
    use std::time::Duration;
    use tokio;
    use tokio_cron_scheduler::{Job, JobScheduler, JobSchedulerError};
    use tokio_postgres::NoTls;

    pub const JOB_REPL_QUEUE_TOPIC: &str = "job_replica";
    pub const JOB_EXE_QUEUE_TOPIC: &str = "job_execution";
    pub const LOCAL_JOB_REPL_QUEUE: &str = "localhost:29092";
    pub const DOCKER_JOB_REPL_QUEUE: &str = "job_replica_queue:9092";
    pub const JOB_REPL_QUEUE_BROKER: &str = DOCKER_JOB_REPL_QUEUE;
    pub const LOCAL_JOB_EXE_QUEUE: &str = "localhost:39092";
    pub const DOCKER_JOB_EXE_QUEUE: &str = "job_execution_queue:9092";
    pub const JOB_EXE_QUEUE_BROKER: &str = DOCKER_JOB_EXE_QUEUE;

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

    pub fn create_kafka_topic_and_resend(topic: &str, value: &str) -> Result<(), kafka::Error> {
        let mut client = KafkaClient::new(vec![JOB_EXE_QUEUE_BROKER.to_owned()]);

        // wait for topic to exist
        while client.load_metadata(&vec![topic.to_owned()]).is_err() {
            std::thread::sleep(Duration::from_secs(2));
        }
        std::thread::sleep(Duration::from_secs(5));

        Producer::from_hosts(vec![JOB_EXE_QUEUE_BROKER.to_owned()])
            .with_ack_timeout(Duration::from_secs(1))
            .with_required_acks(RequiredAcks::One)
            .create()
            .unwrap()
            .send(&Record::from_value(topic, value.as_bytes()))?;
        Ok(())
    }

    pub fn send_to_exe_queue(topic: &str, value: &str) -> Result<(), kafka::Error> {
        let mut producer = Producer::from_hosts(vec![JOB_EXE_QUEUE_BROKER.to_owned()])
            .with_ack_timeout(Duration::from_secs(1))
            .with_required_acks(RequiredAcks::One)
            .create()
            .unwrap();

        let record = Record::from_value(topic, value.as_bytes());
        match producer.send(&record) {
            Ok(_) => Ok(()),
            Err(_) => match create_kafka_topic_and_resend(topic, value) {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            },
        }
    }

    pub struct ShedRSScheduler {
        pub engine: JobScheduler,
    }

    impl ShedRSScheduler {
        pub async fn add_one_shot(
            &self,
            job_id: &str,
            seconds: u64,
            recur: &str,
            job_label: &str,
            job_payload: &str,
        ) -> Result<(), JobSchedulerError> {
            let jid = String::from(job_id);
            let rec = String::from(recur);
            let label = String::from(job_label);
            let payload = String::from(job_payload);

            let _ = init_job_store(&jid, seconds, &recur, &label, &payload).await;

            self.engine
                .add(Job::new_one_shot(
                    Duration::from_secs(seconds),
                    move |_uuid, _l| {
                        let jid = jid.clone();
                        let rec = rec.clone();
                        let lab = label.clone();
                        let pay = payload.clone();
                        tok_rt().spawn(async move {
                            let res = take_job_store(&jid).await;
                            match res {
                                Ok(0) => {
                                    print!("job {} underway .. ", jid);
                                    let _ = send_to_exe_queue(
                                        &lab,
                                        &json!({
                                            "job_id": jid,
                                            "recur": rec,
                                            "job_payload": pay,
                                        })
                                        .to_string(),
                                    );
                                }
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

    pub async fn setup_db(
        is_follower: bool,
    ) -> Result<Vec<(String, String, String, String, String)>, tokio_postgres::Error> {
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
                    cron            VARCHAR NOT NULL DEFAULT 'null',
                    taken           BOOLEAN NOT NULL DEFAULT FALSE,
                    label           VARCHAR NOT NULL DEFAULT 'null',
                    payload         VARCHAR NOT NULL DEFAULT 'null'
                )
            ",
            )
            .await?;

        Ok(match is_follower {
            true => {
                // get all job_id,seconds that are not taken
                client
                    .query(
                        "SELECT job_id, seconds, cron, label, payload FROM jobs WHERE taken = FALSE",
                        &[],
                    )
                    .await?
                    .iter()
                    .map(|row| (row.get(0), row.get(1), row.get(2), row.get(3), row.get(4)))
                    .collect()
            }
            false => vec![],
        })
    }

    pub async fn init_job_store(
        job_id: &str,
        seconds: u64,
        cron: &str,
        label: &str,
        payload: &str,
    ) -> Result<(), tokio_postgres::Error> {
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
                "INSERT INTO jobs (job_id, seconds, cron, label, payload) VALUES ($1, $2, $3, $4, $5) ON CONFLICT (job_id) DO NOTHING",
                &[&job_id, &seconds.to_string(), &cron, &label, &payload],
            )
            .await?;

        Ok(())
    }

    pub async fn take_job_store(job_id: &str) -> Result<i8, tokio_postgres::Error> {
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
}
