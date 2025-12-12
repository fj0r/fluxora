use crate::config::{Queue, QueueIncome, QueueOutgo};
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
            QueueIncome::kafka(income) => {
                let mut income_mq: KafkaManagerIncome<I> = KafkaManagerIncome::new(income);
                income_mq.run().await;
                income_mq.get_rx()
            }
            QueueIncome::iggy(_) => {
                todo!()
            }
        };

        let outgo_tx = match self.outgo {
            QueueOutgo::kafka(outgo) => {
                let mut outgo_mq: KafkaManagerOutgo<O> = KafkaManagerOutgo::new(outgo);
                outgo_mq.run().await;
                outgo_mq.get_tx()
            }
            QueueOutgo::iggy(_) => {
                todo!()
            }
        };

        (outgo_tx, income_rx)
    }
}
