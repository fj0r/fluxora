use crate::config::{Queue, QueueIncome, QueueOutgo};
#[cfg(feature = "iggy")]
use crate::iggy::{IggyManagerIncome, IggyManagerOutgo};
#[cfg(feature = "kafka")]
use crate::kafka::{KafkaManagerIncome, KafkaManagerOutgo};
use crate::queue::{MessageQueue, MessageQueueIncome, MessageQueueOutgo};
use crate::{Event, time::Created};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::{
    Mutex,
    mpsc::{UnboundedReceiver, UnboundedSender},
};

impl MessageQueue for Queue {
    //I Envelope<T>
    //O ChatMessage<T>
    async fn split<I, O>(
        self,
    ) -> (
        Option<UnboundedSender<O>>,
        Option<Arc<Mutex<UnboundedReceiver<I>>>>,
    )
    where
        I: Event<Created> + Send + Serialize + for<'a> Deserialize<'a> + Clone + Debug + 'static,
        O: Event<Created> + Send + Serialize + for<'a> Deserialize<'a> + Clone + Debug + 'static,
    {
        let income_rx = match self.income {
            #[cfg(feature = "kafka")]
            QueueIncome::kafka(income) => {
                let mut income_mq: KafkaManagerIncome<I> = KafkaManagerIncome::new(income);
                let _ = income_mq.run().await;
                income_mq.get_rx()
            }
            #[cfg(feature = "iggy")]
            QueueIncome::iggy(income) => {
                let mut income_mq: IggyManagerIncome<I> = IggyManagerIncome::new(income);
                let _ = income_mq.run().await;
                income_mq.get_rx()
            }
        };

        let outgo_tx = match self.outgo {
            #[cfg(feature = "kafka")]
            QueueOutgo::kafka(outgo) => {
                let mut outgo_mq: KafkaManagerOutgo<O> = KafkaManagerOutgo::new(outgo);
                let _ = outgo_mq.run().await;
                outgo_mq.get_tx()
            }
            #[cfg(feature = "iggy")]
            QueueOutgo::iggy(outgo) => {
                let mut outgo_mq: IggyManagerOutgo<O> = IggyManagerOutgo::new(outgo);
                let _ = outgo_mq.run().await;
                outgo_mq.get_tx()
            }
        };

        (outgo_tx, income_rx)
    }
}
