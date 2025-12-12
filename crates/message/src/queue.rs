use crate::{Event, time::Created};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::{
    Mutex,
    mpsc::{UnboundedReceiver, UnboundedSender},
};

pub trait MessageQueueOutgo {
    type Item: Debug + Send + Serialize + for<'a> Deserialize<'a>;

    #[allow(unused)]
    fn run(&mut self) -> impl std::future::Future<Output = ()> + Send;

    #[allow(unused)]
    fn get_tx(&self) -> Option<UnboundedSender<Self::Item>>;
}

pub trait MessageQueueIncome {
    type Item: Debug + Send + Serialize + for<'a> Deserialize<'a>;

    #[allow(unused)]
    fn run(&mut self) -> impl std::future::Future<Output = ()> + Send;

    #[allow(unused)]
    fn get_rx(&self) -> Option<Arc<Mutex<UnboundedReceiver<Self::Item>>>>;
}

pub trait MessageQueue {
    #[allow(async_fn_in_trait)]
    async fn split<I, O>(
        self,
    ) -> (
        Option<UnboundedSender<O>>,
        Option<Arc<Mutex<UnboundedReceiver<I>>>>,
    )
    where
        I: Event<Created> + Send + Serialize + for<'a> Deserialize<'a> + Clone + Debug + 'static,
        O: Event<Created> + Send + Serialize + for<'a> Deserialize<'a> + Clone + Debug + 'static;
}
