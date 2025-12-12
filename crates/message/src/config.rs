use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct KafkaIncomeConfig {
    pub broker: Vec<String>,
    pub topic: Vec<String>,
    pub group: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct KafkaOutgoConfig {
    pub broker: Vec<String>,
    pub topic: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct IggyIncomeConfig {
    pub broker: String,
    pub stream: String,
    pub topic: String,
    pub group: Option<String>,
}

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
    #[allow(non_camel_case_types)]
    kafka(KafkaIncomeConfig),
    #[allow(non_camel_case_types)]
    iggy(IggyIncomeConfig),
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
#[allow(unused)]
pub enum QueueOutgo {
    #[allow(non_camel_case_types)]
    kafka(KafkaOutgoConfig),
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
