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
            match utils::setup_db(false).await {
                Ok(_) => {
                    println!("DB Setup ... OK");
                    break;
                }
                Err(msg) => println!("ERROR {}", msg),
            }
            std::thread::sleep(Duration::from_secs(3));
        }

        fn send_to_kafka(topic: &str, value: &str) -> Result<(), kafka::Error> {
            let mut producer = Producer::from_hosts(vec![utils::JOB_REPL_QUEUE_BROKER.to_owned()])
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

                let res = send_to_kafka(utils::JOB_REPL_QUEUE_TOPIC, &data.to_string());
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
