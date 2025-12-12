use crate::config;
use crate::time::Created;
use crate::{
    Event,
    queue::{MessageQueueIncome, MessageQueueOutgo},
};
use anyhow::Result;
use config::{KafkaIncomeConfig, KafkaOutgoConfig};
use rdkafka::client::ClientContext;
use rdkafka::config::{ClientConfig, RDKafkaLogLevel};
use rdkafka::consumer::stream_consumer::StreamConsumer;
use rdkafka::consumer::{BaseConsumer, CommitMode, Consumer, ConsumerContext, Rebalance};
use rdkafka::error::KafkaResult;
use rdkafka::message::{Header, Message, OwnedHeaders};
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::topic_partition_list::TopicPartitionList;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::{
    Mutex,
    mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel},
};
use tokio::task::spawn;
use tracing::{error, info, warn};

#[derive(Clone)]
pub struct KafkaManagerOutgo<T>
where
    T: Send + Serialize + for<'de> Deserialize<'de>,
{
    tx: Option<UnboundedSender<T>>,
    producer: KafkaOutgoConfig,
}

impl<T> KafkaManagerOutgo<T>
where
    T: Send + Serialize + for<'de> Deserialize<'de> + 'static,
{
    pub fn new(producer: KafkaOutgoConfig) -> Self {
        Self { tx: None, producer }
    }
}

impl<T> MessageQueueOutgo for KafkaManagerOutgo<T>
where
    T: Debug + Clone + Send + Serialize + for<'de> Deserialize<'de> + 'static,
{
    type Item = T;

    async fn run(&mut self) -> Result<()> {
        let (tx, mut rx) = unbounded_channel::<Self::Item>();
        let cfg = self.producer.clone();

        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", &cfg.broker[0])
            .set("message.timeout.ms", "5000")
            .create()
            .expect("Failed to create Kafka producer");

        spawn(async move {
            // let topic : Vec<&str> = producer_cfg.topic.iter().map(<_>::as_ref).collect();
            while let Some(value) = rx.recv().await {
                let value = serde_json::to_string(&value).expect("serde to string");
                let _delivery_status = producer
                    .send(
                        FutureRecord::to(&cfg.topic)
                            .payload(&value)
                            .key("")
                            .headers(OwnedHeaders::new().insert(Header {
                                key: "",
                                value: Some(""),
                            })),
                        Duration::from_secs(0),
                    )
                    .await;
            }
        });

        self.tx = Some(tx);
        Ok(())
    }

    fn get_tx(&self) -> Option<UnboundedSender<Self::Item>> {
        self.tx.clone()
    }
}

#[derive(Clone)]
pub struct KafkaManagerIncome<T>
where
    T: Send + Serialize + for<'de> Deserialize<'de>,
{
    rx: Option<Arc<Mutex<UnboundedReceiver<T>>>>,
    consumer: KafkaIncomeConfig,
}

impl<T> KafkaManagerIncome<T>
where
    T: Send + Serialize + for<'de> Deserialize<'de> + 'static,
{
    pub fn new(consumer: KafkaIncomeConfig) -> Self {
        Self { rx: None, consumer }
    }
}

impl<T> MessageQueueIncome for KafkaManagerIncome<T>
where
    T: Debug + Clone + Send + Serialize + for<'de> Deserialize<'de> + Event<Created> + 'static,
{
    type Item = T;

    async fn run(&mut self) -> Result<()> {
        let (tx, rx) = unbounded_channel::<Self::Item>();
        let cfg = self.consumer.clone();

        let context = CustomContext;

        let consumer: LoggingConsumer = ClientConfig::new()
            .set("group.id", cfg.group.unwrap_or("default".into()))
            .set("bootstrap.servers", cfg.broker[0].clone())
            .set("enable.partition.eof", "false")
            .set("session.timeout.ms", "6000")
            .set("enable.auto.commit", "true")
            //.set("statistics.interval.ms", "30000")
            //.set("auto.offset.reset", "smallest")
            .set_log_level(RDKafkaLogLevel::Debug)
            .create_with_context(context)
            .expect("Failed to create Kafka consumer");

        spawn(async move {
            let topic: Vec<&str> = cfg.topic.iter().map(<_>::as_ref).collect();

            consumer
                .subscribe(topic.as_slice())
                .expect("Can't subscribe to specified topics");
            loop {
                match consumer.recv().await {
                    Err(e) => warn!("Kafka error: {}", e),
                    Ok(m) => {
                        let payload = match m.payload_view::<str>() {
                            None => "",
                            Some(Ok(s)) => s,
                            Some(Err(e)) => {
                                warn!("Error while deserializing message payload: {:?}", e);
                                ""
                            }
                        };

                        match serde_json::from_str::<Self::Item>(payload) {
                            Ok(mut value) => {
                                value.set_time(m.timestamp().into());
                                if let Err(e) = tx.send(value) {
                                    error!("Failed to send message from consumer: {}", e);
                                }
                            }
                            Err(e) => {
                                error!("Failed to deserialize: {:?}", e);
                            }
                        }
                        /*
                        info!("key: '{:?}', payload: '{}', topic: {}, partition: {}, offset: {}, timestamp: {:?}",
                              m.key(), payload, m.topic(), m.partition(), m.offset(), m.timestamp());
                        if let Some(headers) = m.headers() {
                            for header in headers.iter() {
                                info!("  Header {:#?}: {:?}", header.key, header.value);
                            }
                        }
                        */
                        consumer.commit_message(&m, CommitMode::Async).unwrap();
                    }
                }
            }
        });
        self.rx = Some(Arc::new(Mutex::new(rx)));
        Ok(())
    }

    fn get_rx(&self) -> Option<Arc<Mutex<UnboundedReceiver<Self::Item>>>> {
        self.rx.clone()
    }
}

struct CustomContext;

impl ClientContext for CustomContext {}

impl ConsumerContext for CustomContext {
    fn pre_rebalance(&self, _: &BaseConsumer<Self>, rebalance: &Rebalance) {
        info!("Pre rebalance {:?}", rebalance);
    }

    fn post_rebalance(&self, _: &BaseConsumer<Self>, rebalance: &Rebalance) {
        info!("Post rebalance {:?}", rebalance);
    }

    fn commit_callback(&self, result: KafkaResult<()>, _offsets: &TopicPartitionList) {
        info!("Committing offsets: {:?}", result);
    }
}

type LoggingConsumer = StreamConsumer<CustomContext>;
