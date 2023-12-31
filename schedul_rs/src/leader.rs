pub mod leader {
    use crate::utils;
    use chrono;
    use cron;
    use kafka::producer::{Producer, Record, RequiredAcks};
    use nickel::{JsonBody, Nickel};
    use serde_json::json;
    use std::str::FromStr;
    use std::time::Duration;
    use tokio_cron_scheduler::JobScheduler;
    use uuid::Uuid;

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

        fn send_to_replica_queue(topic: &str, value: &str) -> Result<(), kafka::Error> {
            let mut producer = Producer::from_hosts(vec![utils::JOB_REPL_QUEUE_BROKER.to_owned()])
                .with_ack_timeout(Duration::from_secs(1))
                .with_required_acks(RequiredAcks::One)
                .create()
                .unwrap();

            let record = Record::from_value(topic, value.as_bytes());
            let res = producer.send(&record)?;
            Ok(res)
        }

        fn schedule_instruction_to_seconds(job_type: &str, schedule_value: &str) -> u64 {
            let s = match job_type {
                "seconds" => schedule_value.parse::<u64>().unwrap(),
                "timestamp" => {
                    let now = chrono::Utc::now().timestamp() as f64;
                    let end = schedule_value.parse::<f64>().unwrap();
                    let delta = end - now;
                    delta as u64
                }
                "datetime" => {
                    let now = chrono::Utc::now();
                    let end = chrono::DateTime::parse_from_rfc3339(schedule_value).unwrap();
                    let delta = end.timestamp() - now.timestamp();
                    delta as u64
                }
                "cron" => {
                    let cron_sched = cron::Schedule::from_str(schedule_value).unwrap();
                    let now = chrono::Utc::now();
                    let next = cron_sched.upcoming(chrono::Utc).next().unwrap();
                    let delta = next.timestamp() - now.timestamp();
                    delta as u64
                }
                _ => panic!("Invalid job_type"),
            };

            match s {
                s if s > 0 => s,
                _ => 0,
            }
        }

        server.utilize(router! {
            get utils::v1("health") => |_req, _res| {
                r#"{ "status" : "ok" }"#
            }

            post utils::v1("job") => |req, _res| {
                let data = req.json_as::<serde_json::Value>().unwrap();
                let backup_id = Uuid::new_v4().to_string();
                let job_id = data["schedule_id"].as_str().unwrap_or(&backup_id);
                let schedule_type = data["schedule_type"].as_str().unwrap_or("null");
                let schedule_value = data["schedule_value"].as_str().unwrap_or("null");
                let seconds = schedule_instruction_to_seconds(schedule_type, schedule_value);
                let job_label = data["job_label"].as_str().unwrap_or("null");
                let job_payload = data["job_payload"].as_str().unwrap_or("null");
                let recur = match schedule_type {
                    "cron" => schedule_value,
                    _ => "null",
                };

                let _ = utils::tok_rt().block_on(sched.add_one_shot(job_id, seconds, recur, job_label, job_payload));

                let replica_message = json!({
                    "job_id": job_id,
                    "seconds": seconds,
                    "recur": recur,
                    "job_label": job_label,
                    "job_payload": job_payload,
                }).to_string();
                println!(
                    "job -> {}",
                    replica_message
                );
                let res = send_to_replica_queue(utils::JOB_REPL_QUEUE_TOPIC, replica_message.to_string().as_str());

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
