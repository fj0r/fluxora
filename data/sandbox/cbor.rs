#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! serde_cbor = "0.11"
//! bincode = { version = "2.0.1", features = ["serde"] }
//! serde_json = "1.0.145"
//! prost = "0.12"
//! serde = { version = "1.0", features = ["derive"] }
//! ```

use serde::{Deserialize, Serialize};
use serde_cbor::{from_slice, to_vec};
use serde_json::{from_str, to_string, Map, Value};
use std::collections::HashMap;
use std::fmt::Debug;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let o = r#"{"attrs":{"class":["maximize"]},"type":"case","children":[{"type":"text","bind":{"value":{"default":"**chat**","source":"login"}},"attrs":{"class":["--header","nogrow"],"format":"md"}},{"type":"case","attrs":{"horizontal":true,"class":["--body","shrink","border","round"]},"children":[{"type":"case","attrs":{"class":["--leftside","nogrow"]},"children":[{"type":"text","attrs":{"format":"md","class":["box"]},"bind":{"value":{"default":"**channel**"}}},{"type":"select","attrs":{"class":["--channel","ji-center","nogrow"]},"bind":{"value":{"event":"channel::select","default":"2"},"options":{"source":"channel::list"}}}]},{"type":"case","attrs":{"class":["main","shrink"]},"children":[{"type":"case","children":[{"type":"rack","attrs":{"class":["--message","box","ac-start","gap","scrolly"],"scroll":true},"bind":{"value":{"source":"chat"}},"item":[{"type":"case","attrs":{"class":["nogrow","as-end","accent","box","border","round","shadow"],"selector":"ask"}},{"type":"case","attrs":{"class":["nogrow","as-stretch","box","border","round","shadow"]}}]},{"type":"float","attrs":{"bottom":"0.2rem","left":"1rem"},"children":[{"type":"popup","attrs":{"class":["--cart","box","border","shadow","primary"],"direction":"U"},"bind":{"value":{"source":"cart"}},"children":[{"type":"render","name":"cart","data":{}},{"type":"case","children":[{"type":"text","bind":{"value":{"default":"a"}}},{"type":"text","bind":{"value":{"default":"b"}}},{"type":"text","bind":{"value":{"default":"c"}}}]}]}]}]},{"type":"case","attrs":{"class":["--input","nogrow","box"],"horizontal":true},"children":[{"type":"case","attrs":{"horizontal":true},"children":[{"type":"input","bind":{"value":{"event":"message"}}}]}]}]}]}]}"#;
    let origin = from_str::<Brick>(&o)?;

    let cbor_bytes = to_vec(&origin)?;
    let decoded_user: Brick = from_slice(&cbor_bytes)?;
    println!("CBOR Decode: {:?}", decoded_user);
    println!("CBOR size: {} bytes", cbor_bytes.len());

    let json_str = to_string(&origin)?;
    let json_bytes = json_str.as_bytes();
    println!("JSON str size: {} bytes", json_str.len());

    // let bincode_bytes = bincode::encode_to_vec(&origin, bincode::config::standard())?;
    // println!("BINCODE bytes: {:?}", bincode_bytes);
    // println!("BINCODE size: {} bytes", bincode_bytes.len());
    // let decoded_user: Brick =
    //     bincode::decode_from_slice(&bincode_bytes, bincode::config::standard())?;
    // println!("BINCODE Decode: {:?}", decoded_user);

    Ok(())
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub enum JsType {
    #[allow(non_camel_case_types)]
    bool,
    #[allow(non_camel_case_types)]
    number,
    #[default]
    #[allow(non_camel_case_types)]
    text,
    #[allow(non_camel_case_types)]
    password,
    #[allow(non_camel_case_types)]
    button,
    #[allow(non_camel_case_types)]
    submit,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BindVariant {
    Source {
        source: String,
    },
    Target {
        target: String,
    },
    Event {
        event: String,
    },
    Field {
        field: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        payload: Option<Value>,
    },
    Submit {
        submit: bool,
    },
    Default {},
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Bind {
    #[serde(flatten)]
    pub variant: BindVariant,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<JsType>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ClassAttr {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub class: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct SizeAttr {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub class: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PosH {
    #[allow(non_camel_case_types)]
    left(String),
    #[allow(non_camel_case_types)]
    right(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PosV {
    #[allow(non_camel_case_types)]
    top(String),
    #[allow(non_camel_case_types)]
    bottom(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct PositionAttr {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub class: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub h: Option<PosH>,
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub v: Option<PosV>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Direction {
    U,
    D,
    L,
    R,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct DirectionAttr {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub class: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<Direction>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct StyleAttr {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub class: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Placeholder {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attrs: Option<ClassAttr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bind: Option<HashMap<String, Bind>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<Brick>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Chart {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attrs: Option<ClassAttr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bind: Option<HashMap<String, Bind>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Diagram {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attrs: Option<ClassAttr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bind: Option<HashMap<String, Bind>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Float {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attrs: Option<PositionAttr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bind: Option<HashMap<String, Bind>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<Brick>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct FoldAttr {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub class: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replace_header: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub float_body: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Fold {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attrs: Option<FoldAttr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bind: Option<HashMap<String, Bind>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<Brick>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item: Option<Vec<Brick>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct FormAttr {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub class: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instant: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Form {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attrs: Option<FormAttr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bind: Option<HashMap<String, Bind>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<Brick>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Popup {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attrs: Option<DirectionAttr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bind: Option<HashMap<String, Bind>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<Brick>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Svg {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attrs: Option<SizeAttr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bind: Option<HashMap<String, Bind>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<Brick>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Group {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attrs: Option<StyleAttr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bind: Option<HashMap<String, Bind>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<Brick>>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Path {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attrs: Option<ClassAttr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bind: Option<HashMap<String, Bind>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct RackAttr {
    #[serde(default)]
    pub scroll: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub class: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Rack {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attrs: Option<RackAttr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bind: Option<HashMap<String, Bind>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<Brick>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item: Option<Vec<Brick>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ButtonAttr {
    #[serde(default)]
    pub oneshot: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub class: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Button {
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attrs: Option<ButtonAttr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bind: Option<HashMap<String, Bind>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ImageAttr {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub class: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub desc: Option<String>,
    #[serde(default)]
    pub thumb: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Image {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attrs: Option<ImageAttr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bind: Option<HashMap<String, Bind>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Input {
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attrs: Option<ClassAttr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bind: Option<HashMap<String, Bind>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Select {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attrs: Option<ClassAttr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bind: Option<HashMap<String, Bind>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<Brick>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Table {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<Brick>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Thead {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<Brick>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Tbody {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<Brick>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Tr {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<Brick>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Th {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<Brick>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Td {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<Brick>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct TextAttr {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub class: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Text {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attrs: Option<TextAttr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bind: Option<HashMap<String, Bind>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct TextArea {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attrs: Option<ClassAttr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bind: Option<HashMap<String, Bind>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct CaseAttr {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub class: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub horizontal: Option<bool>,
    #[allow(non_camel_case_types)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grid: Option<Map<String, Value>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Case {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attrs: Option<CaseAttr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bind: Option<HashMap<String, Bind>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<Brick>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Render {
    name: String,
    data: Map<String, Value>,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Brick {
    case(Case),
    placeholder(Placeholder),
    chart(Chart),
    diagram(Diagram),
    float(Float),
    fold(Fold),
    form(Form),
    popup(Popup),
    svg(Svg),
    group(Group),
    path(Path),
    rack(Rack),
    button(Button),
    image(Image),
    input(Input),
    select(Select),
    table(Table),
    thead(Thead),
    tbody(Tbody),
    tr(Tr),
    th(Th),
    td(Td),
    text(Text),
    textarea(TextArea),
    render(Render),
}
