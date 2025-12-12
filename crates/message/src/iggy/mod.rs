use crate::config;
use crate::time::Created;
use crate::{
    Event,
    queue::{MessageQueueIncome, MessageQueueOutgo},
};
use anyhow::Result;
use config::{IggyIncomeConfig, IggyOutgoConfig};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::str::FromStr;
use std::sync::Arc;

use iggy::prelude::{
    AutoCommit::IntervalOrWhen, AutoCommitWhen::ConsumingAllMessages, DirectConfig, IggyClient,
    IggyDuration, IggyMessage, Partitioning, PollingStrategy,
};
use tokio::sync::{
    Mutex,
    mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel},
};
use tokio::task::spawn;
use tracing::{error, info, warn};

#[derive(Clone)]
pub struct IggyManagerOutgo<T>
where
    T: Send + Serialize + for<'de> Deserialize<'de>,
{
    tx: Option<UnboundedSender<T>>,
    producer: IggyOutgoConfig,
}

impl<T> IggyManagerOutgo<T>
where
    T: Send + Serialize + for<'de> Deserialize<'de> + 'static,
{
    pub fn new(producer: IggyOutgoConfig) -> Self {
        Self { tx: None, producer }
    }
}

impl<T> MessageQueueOutgo for IggyManagerOutgo<T>
where
    T: Debug + Clone + Send + Serialize + for<'de> Deserialize<'de> + 'static,
{
    type Item = T;

    async fn run(&mut self) -> Result<()> {
        let (tx, mut producer_rx) = unbounded_channel::<Self::Item>();
        let cfg = &self.producer;
        let client = IggyClient::from_connection_string(&cfg.broker)?;

        let producer = client
            .producer(&cfg.stream, &cfg.topic)?
            .direct(
                DirectConfig::builder()
                    .batch_length(1000)
                    .linger_time(IggyDuration::from_str("1ms")?)
                    .build(),
            )
            .partitioning(Partitioning::balanced())
            .build();

        producer.init().await?;

        spawn(async move {
            // let topic : Vec<&str> = producer_cfg.topic.iter().map(<_>::as_ref).collect();
            while let Some(value) = producer_rx.recv().await {
                let value = serde_json::to_string(&value).expect("serde to string");
                producer.send(vec![IggyMessage::from(value)]).await;
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
pub struct IggyManagerIncome<T>
where
    T: Send + Serialize + for<'de> Deserialize<'de>,
{
    rx: Option<Arc<Mutex<UnboundedReceiver<T>>>>,
    consumer: IggyIncomeConfig,
}

impl<T> IggyManagerIncome<T>
where
    T: Send + Serialize + for<'de> Deserialize<'de> + 'static,
{
    pub fn new(consumer: IggyIncomeConfig) -> Self {
        Self { rx: None, consumer }
    }
}

impl<T> MessageQueueIncome for IggyManagerIncome<T>
where
    T: Debug + Clone + Send + Serialize + for<'de> Deserialize<'de> + Event<Created> + 'static,
{
    type Item = T;

    async fn run(&mut self) -> Result<()> {
        let (tx, rx) = unbounded_channel::<Self::Item>();
        let cfg = &self.consumer;
        let client = IggyClient::from_connection_string(&cfg.broker)?;
        let mut consumer = client
            .consumer_group(
                cfg.group.as_ref().map_or("default", |v| v),
                &cfg.stream,
                &cfg.topic,
            )?
            .auto_commit(IntervalOrWhen(
                IggyDuration::from_str("1s")?,
                ConsumingAllMessages,
            ))
            .create_consumer_group_if_not_exists()
            .auto_join_consumer_group()
            .polling_strategy(PollingStrategy::next())
            .poll_interval(IggyDuration::from_str("1ms")?)
            .batch_length(1000)
            .build();
        consumer.init().await?;

        spawn(async move {
            /* FIXME:
            while let Some(m) = consumer.next().await {
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
            }
            */
        });
        self.rx = Some(Arc::new(Mutex::new(rx)));
        Ok(())
    }

    fn get_rx(&self) -> Option<Arc<Mutex<UnboundedReceiver<Self::Item>>>> {
        self.rx.clone()
    }
}
