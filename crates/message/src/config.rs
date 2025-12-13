use serde::Deserialize;

#[cfg(feature = "kafka")]
#[derive(Debug, Deserialize, Clone)]
pub struct KafkaIncomeConfig {
    pub broker: Vec<String>,
    pub topic: Vec<String>,
    pub group: Option<String>,
}

#[cfg(feature = "kafka")]
#[derive(Debug, Deserialize, Clone)]
pub struct KafkaOutgoConfig {
    pub broker: Vec<String>,
    pub topic: String,
}

#[cfg(feature = "iggy")]
#[derive(Debug, Deserialize, Clone)]
pub struct IggyIncomeConfig {
    pub broker: String,
    pub stream: String,
    pub topic: String,
    pub group: Option<String>,
}

#[cfg(feature = "iggy")]
#[derive(Debug, Deserialize, Clone)]
pub struct IggyOutgoConfig {
    pub broker: String,
    pub stream: String,
    pub topic: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
#[allow(unused)]
pub enum QueueIncome {
    #[cfg(feature = "kafka")]
    #[allow(non_camel_case_types)]
    kafka(KafkaIncomeConfig),
    #[cfg(feature = "iggy")]
    #[allow(non_camel_case_types)]
    iggy(IggyIncomeConfig),
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
#[allow(unused)]
pub enum QueueOutgo {
    #[cfg(feature = "kafka")]
    #[allow(non_camel_case_types)]
    kafka(KafkaOutgoConfig),
    #[cfg(feature = "iggy")]
    #[allow(non_camel_case_types)]
    iggy(IggyOutgoConfig),
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Queue {
    #[serde(default)]
    pub disable: bool,
    pub outgo: QueueOutgo,
    pub income: QueueIncome,
}
