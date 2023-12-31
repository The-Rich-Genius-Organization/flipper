pub mod redoer {
    use crate::utils;
    use kafka::consumer::{Consumer, FetchOffset, GroupOffsetStorage};
    use reqwest::Client;
    use sha2::{Digest, Sha256};
    use std::collections::HashMap;
    use std::thread::sleep;
    use std::time::Duration;

    fn sha_hash(job_id: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(job_id);
        let result = hasher.finalize();
        format!("{:x}", result)[..8].to_owned()
    }

    async fn redo_job(job_id: &str, recur: &str) -> Result<(), Box<dyn std::error::Error>> {
        let body = HashMap::from([
            ("job_id", sha_hash(job_id)),
            ("schedule_type", String::from("cron")),
            ("schedule_value", String::from(recur)),
        ]);
        Client::new()
            .post("http://lead_loadbalance:5000/api/v1/job")
            .json(&body)
            .send()
            .await?;
        Ok(())
    }

    fn connect_to_kafka() -> Result<Consumer, kafka::Error> {
        let consumer = Consumer::from_hosts(vec![utils::JOB_EXE_QUEUE_BROKER.to_owned()])
            .with_fallback_offset(FetchOffset::Earliest)
            .with_topic(utils::JOB_EXE_QUEUE_TOPIC.to_owned())
            .with_group("job_replica_group".to_owned())
            .with_offset_storage(Some(GroupOffsetStorage::Kafka))
            .create()?;
        Ok(consumer)
    }

    pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
        let mut consumer: Option<Consumer> = None;

        print!("Starting redoer ... ");
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

        let c = consumer.as_mut().unwrap();
        loop {
            let mss = c.poll().unwrap();
            for ms in mss.iter() {
                for m in ms.messages() {
                    let data: serde_json::Value = serde_json::from_slice(m.value).unwrap();
                    let job_id = data["job_id"].as_str().unwrap_or("null");
                    let recur = data["recur"].as_str().unwrap_or("null");

                    if recur != "null" {
                        println!("Redoing job {}", job_id);
                        let _ = redo_job(job_id, recur).await;
                    } // println!(
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
