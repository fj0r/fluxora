use brick::{Bind, BindVariant, Brick, ButtonAttr, Case, Input, Text, TextAttr};
#[cfg(feature = "scratch")]
use maplit::hashmap;
use serde_json::{from_str, to_string};
use std::fs::read_to_string;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let a = Brick::case(Case {
        sub: Some(vec![
            Brick::text(Text {
                attrs: Some(TextAttr {
                    format: Some("md".to_string()),
                    class: Some(vec!["f".to_string()]),
                }),
                ..Default::default()
            }),
            Brick::input(Input {
                bind: Some(hashmap! {
                    "value".to_owned() => Bind {
                        variant: BindVariant::Default {},
                        default: Some(from_str("\"test\"")?),
                        ..Default::default()
                    }
                }),
                ..Default::default()
            }),
        ]),
        ..Default::default()
    });
    println!("{:?}", to_string(&a));

    let chat_layout = read_to_string("brick/examples/layout.json")?;
    let chat_layout: Brick = from_str(&chat_layout)?;
    println!("{:?}", &chat_layout);

    Ok(())
}
