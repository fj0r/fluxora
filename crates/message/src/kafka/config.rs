use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct QueueIncomeConfig {
    pub broker: Vec<String>,
    pub topic: Vec<String>,
    pub group: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
#[allow(unused)]
pub enum QueueIncome {
    #[allow(non_camel_case_types)]
    kafka(QueueIncomeConfig),
}

#[derive(Debug, Deserialize, Clone)]
pub struct QueueOutgoConfig {
    pub broker: Vec<String>,
    pub topic: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
#[allow(unused)]
pub enum QueueOutgo {
    #[allow(non_camel_case_types)]
    kafka(QueueOutgoConfig),
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Queue {
    #[serde(default)]
    pub disable: bool,
    pub outgo: QueueOutgo,
    pub income: QueueIncome,
}
