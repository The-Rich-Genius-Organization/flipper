const kafka = require("kafka-node");

const KAFKA_URL = "job_execution_queue:9092";
const TOPIC = "email";

const sleep = (time) =>
  new Promise((res) => setTimeout(res, time, "done sleeping"));

const kafkaCheckTopicExists = async (client, topic) => {
  // wait until kafka topic exists
  while (true) {
    const topics = await new Promise((resolve, reject) => {
      client.loadMetadataForTopics([], (err, resp) => {
        resolve(resp);
      });
    });
    if (topics[1].metadata[topic]) break;
    await sleep(1000);
  }
};

const main = async () => {
  const kafkaClient = new kafka.KafkaClient({ kafkaHost: KAFKA_URL });
  await kafkaCheckTopicExists(kafkaClient, TOPIC);

  const consumer = new kafka.Consumer(kafkaClient, [{ topic: TOPIC }]);

  consumer.on("message", async (message) => {
    await sleep(500); // just for demo purposes
    console.log(`doing something with job: ${message.value}`);
  });
};

main();
