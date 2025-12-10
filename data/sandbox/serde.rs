//! ```cargo
//! [dependencies]
//! serde_json = "1.0.140"
//! serde = { version = "1.0.219", features = ["derive"] }
//! serde_json_path = "0.7.2"
//! ```

use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_json_path::JsonPath;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Layout<'a> {
    #[serde(rename = "type")]
    pub kind: &'a str,
    pub data: Option<&'a str>,
    pub item: Option<Vec<Box<Layout<'a>>>>,
    pub children: Option<Vec<Box<Layout<'a>>>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Empty;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InfluxTmpl<'a> {
    pub name: &'a str,
    pub data: &'a str,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Target {
    Map,
    List,
}

impl Default for Target {
    fn default() -> Self {
        Self::Map
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Influx<'a> {
    pub event: &'a str,
    pub data: Layout<'a>,
    #[serde(default)]
    pub kind: Target,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(bound(deserialize = "'de: 'a"))]
#[serde(tag = "action")]
pub enum Content<'a> {
    #[allow(non_camel_case_types)]
    create(Influx<'a>),

    #[allow(non_camel_case_types)]
    tmpl(InfluxTmpl<'a>),

    #[allow(non_camel_case_types)]
    merge(Influx<'a>),

    #[allow(non_camel_case_types)]
    join(Influx<'a>),

    #[allow(non_camel_case_types)]
    #[default]
    empty,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let s = r#"
    {
      "action": "create",
      "event": "init",
      "data": {
        "type": "box",
        "children": [
          {
            "type": "header",
            "title": "test"
          },
          {
            "type": "scroll",
            "data": "chat",
            "item": [
              {
                "type": "card"
              }
            ]
          },
          {
            "type": "input",
            "data": "message"
          }
        ]
      }
    }"#;
    let a = serde_json::from_str::<Content>(&s)?;
    println!("de ==> {:#?}", a);
    println!("ser ==> {:#?}", serde_json::to_string::<Content>(&a)?);
    let p = JsonPath::parse("$.data.children[0].type")?;
    let x = serde_json::from_str(&s)?;
    let x = p.query(&x).exactly_one();
    println!("query ==> {:#?}", x);
    Ok(())
}
