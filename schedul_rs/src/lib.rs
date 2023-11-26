#[macro_use]
extern crate nickel;

pub mod utils {
    pub const KAFKA_TOPIC: &str = "job_replica";
    pub const LOCAL_KAFKA: &str = "localhost:29092";
    pub const DOCKER_KAFKA: &str = "job_replica_queue:9092";

    pub fn v1(path: &str) -> String {
        format!("/api/v1/{}", path)
    }
}

pub mod leader {
    use crate::utils;
    use kafka::producer::{Producer, Record, RequiredAcks};
    use nickel::Nickel;
    use std::future::Future;
    use std::time::Duration;

    pub fn run() -> impl Future<Output = ()> {
        fn send_to_kafka(topic: &str, value: &str) -> Result<(), kafka::Error> {
            let mut producer = Producer::from_hosts(vec![utils::LOCAL_KAFKA.to_owned()])
                .with_ack_timeout(Duration::from_secs(1))
                .with_required_acks(RequiredAcks::One)
                .create()
                .unwrap();

            let record = Record::from_value(topic, value.as_bytes());
            let res = producer.send(&record)?;
            Ok(res)
        }

        let mut server = Nickel::new();

        server.utilize(router! {
            get utils::v1("health") => |_req, _res| {
                let buf = r#"{ "job_id" : "1234" }"#;
                let res = send_to_kafka(utils::KAFKA_TOPIC, buf);
                match res {
                    Ok(_) => r#"{ "status" : "ok" }"#,
                    _ => r#"{ "status" : "error" }"#,
                }
            }

        });

        server.listen("127.0.0.1:6767").unwrap();
        async {}
    }
}

pub mod follower {
    use crate::utils;
    use kafka::consumer::{Consumer, FetchOffset, GroupOffsetStorage};

    pub fn run() {
        print!("Starting consumer... ");

        let consumer = Consumer::from_hosts(vec![utils::LOCAL_KAFKA.to_owned()])
            .with_topic(utils::KAFKA_TOPIC.to_owned())
            .with_fallback_offset(FetchOffset::Earliest)
            .with_offset_storage(Some(GroupOffsetStorage::Kafka))
            .with_group("job_replica".to_owned())
            .create();
        match consumer {
            Ok(mut c) => {
                println!("OK");
                loop {
                    let mss = c.poll().unwrap();
                    for ms in mss.iter() {
                        for m in ms.messages() {
                            println!(
                                "{}",
                                std::str::from_utf8(m.value)
                                    .unwrap_or("Couldn't decode job replica msg.")
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
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
}
