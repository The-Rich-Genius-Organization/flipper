pub mod follower {
    use crate::utils;
    use kafka::consumer::{Consumer, FetchOffset, GroupOffsetStorage};
    use serde_json::json;
    use std::thread::sleep;
    use std::time::Duration;
    use tokio_cron_scheduler::JobScheduler;

    fn connect_to_kafka() -> Result<Consumer, kafka::Error> {
        let consumer = Consumer::from_hosts(vec![utils::JOB_REPL_QUEUE_BROKER.to_owned()])
            .with_fallback_offset(FetchOffset::Earliest)
            .with_topic(utils::JOB_REPL_QUEUE_TOPIC.to_owned())
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
            match utils::setup_db(true).await {
                Ok(untaken_jobs) => {
                    // on startup, followers should schedule all non-taken jobs
                    print!("DB Setup ... OK ");
                    println!("{:?}", untaken_jobs);
                    for (job_id, seconds, cron, label, payload) in untaken_jobs {
                        let _ = sched
                            .add_one_shot(
                                &job_id,
                                seconds.parse::<u64>().unwrap() + delay,
                                &cron,
                                &label,
                                &payload,
                            )
                            .await;
                    }
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
                    let recur = data["recur"].as_str().unwrap_or("null");
                    let job_label = data["job_label"].as_str().unwrap_or("null");
                    let job_payload = data["job_payload"].as_str().unwrap_or("null");

                    sched
                        .add_one_shot(job_id, seconds, recur, job_label, job_payload)
                        .await
                        .unwrap();
                    println!(
                        "job <- {}",
                        json!({ "job_id": job_id,
                            "seconds": seconds,
                            "recur": recur,
                            "job_label": job_label,
                            "job_payload": job_payload,
                        })
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
