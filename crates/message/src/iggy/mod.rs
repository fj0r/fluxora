use crate::config;
use crate::time::Created;
use crate::{
    Event,
    queue::{MessageQueueIncome, MessageQueueOutgo},
};
use anyhow::{Ok as Okk, Result, anyhow};
use config::{IggyIncomeConfig, IggyOutgoConfig};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::str::FromStr;
use std::sync::Arc;

use futures_util::StreamExt;
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

#[derive(Clone, Debug)]
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
        let client = IggyClient::from_connection_string(&cfg.to_conn())?;

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

        match producer.init().await {
            Ok(_) => {}
            Err(e) => {
                println!("IggyOutgoError: {:#?}", e);
                panic!()
            }
        };

        spawn(async move {
            // let topic : Vec<&str> = producer_cfg.topic.iter().map(<_>::as_ref).collect();
            while let Some(value) = producer_rx.recv().await {
                let value = serde_json::to_string(&value).expect("serde to string");
                let _ = producer.send(vec![IggyMessage::from(value)]).await;
            }
        });

        self.tx = Some(tx);
        Ok(())
    }

    fn get_tx(&self) -> Option<UnboundedSender<Self::Item>> {
        self.tx.clone()
    }
}

#[derive(Clone, Debug)]
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
        let client = IggyClient::from_connection_string(&cfg.to_conn())?;
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
        match consumer.init().await {
            Ok(_) => {}
            Err(e) => {
                println!("IggyIncomeError: {:#?}", e);
                panic!()
            }
        };

        spawn(async move {
            while let Some(Ok(m)) = consumer.next().await {
                let payload = if let Ok(m) = std::str::from_utf8(&m.message.payload) {
                    Ok(m)
                } else {
                    Err(anyhow!("Error while handling message"))
                };
                match serde_json::from_str::<Self::Item>(payload?) {
                    Ok(mut value) => {
                        value.set_time(m.message.header.timestamp.into());
                        if let Err(e) = tx.send(value) {
                            error!("Failed to send message from consumer: {}", e);
                        }
                    }
                    Err(e) => {
                        error!("Failed to deserialize: {:?}", e);
                    }
                }
            }
            Okk(())
        });
        self.rx = Some(Arc::new(Mutex::new(rx)));
        Ok(())
    }

    fn get_rx(&self) -> Option<Arc<Mutex<UnboundedReceiver<Self::Item>>>> {
        self.rx.clone()
    }
}
