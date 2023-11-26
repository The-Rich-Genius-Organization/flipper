#[macro_use]
extern crate nickel;

pub mod utils {
    pub const KAFKA_TOPIC: &str = "job_replica";
    pub const LOCAL_KAFKA: &str = "localhost:29092";
    pub const DOCKER_KAFKA: &str = "job_replica_queue:9092";
    pub const KAFKA_BROKER: &str = DOCKER_KAFKA;

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
            let mut producer = Producer::from_hosts(vec![utils::KAFKA_BROKER.to_owned()])
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
                println!("job <- {}", buf);
                let res = send_to_kafka(utils::KAFKA_TOPIC, buf);
                match res {
                    Ok(_) => r#"{ "status" : "ok" }"#,
                    _ => r#"{ "status" : "error" }"#,
                }
            }

        });

        server.listen("0.0.0.0:6767").unwrap();
        async {}
    }
}

pub mod follower {
    use crate::utils;
    use kafka::consumer::{Consumer, FetchOffset, GroupOffsetStorage};
    use std::thread::sleep;
    use std::time::Duration;

    fn connect_to_kafka() -> Result<Consumer, kafka::Error> {
        let consumer = Consumer::from_hosts(vec![utils::KAFKA_BROKER.to_owned()])
            .with_topic(utils::KAFKA_TOPIC.to_owned())
            .with_fallback_offset(FetchOffset::Earliest)
            .with_group("job_replica_group".to_owned())
            .with_offset_storage(Some(GroupOffsetStorage::Kafka))
            .create()?;
        Ok(consumer)
    }

    pub fn run() {
        print!("Starting consumer ... ");

        let mut consumer: Option<Consumer> = None;
        while consumer.is_none() {
            consumer = match connect_to_kafka() {
                Ok(c) => Some(c),
                _ => None,
            };
            sleep(Duration::from_secs(3));
        }

        println!("OK");
        let c = consumer.as_mut().unwrap();
        loop {
            let mss = c.poll().unwrap();
            for ms in mss.iter() {
                for m in ms.messages() {
                    println!(
                        "job -> {}",
                        std::str::from_utf8(m.value).unwrap_or("Couldn't decode job replica msg.")
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
