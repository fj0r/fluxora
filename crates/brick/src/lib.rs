#[cfg(feature = "dioxus")]
use dioxus::prelude::*;
#[cfg(feature = "schema")]
use schemars::JsonSchema;
#[cfg(feature = "classify")]
pub mod classify;
#[cfg(feature = "classify")]
use classify::Classify;
#[cfg(feature = "merge")]
pub mod merge;
#[cfg(feature = "render")]
pub mod render;
#[cfg(feature = "ops")]
use brick_macro::BrickOps;
#[cfg(feature = "classify")]
use brick_macro::{ClassifyAttrs, ClassifyBrick, ClassifyVariant};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, to_value};
use std::collections::HashMap;
use std::fmt::Debug;

#[cfg(feature = "ops")]
pub trait BrickOps {
    fn get_type(&self) -> &str;
    fn borrow_sub(&self) -> Option<&Vec<Brick>>;
    fn borrow_sub_mut(&mut self) -> Option<&mut Vec<Brick>>;
    fn set_sub(&mut self, brick: Vec<Brick>);
    fn borrow_attrs(&self) -> Option<&dyn Classify>;
    fn borrow_attrs_mut(&mut self) -> Option<&mut dyn Classify>;
    fn get_bind(&self) -> Option<&HashMap<String, Bind>>;
    fn set_bind(&mut self, bind: Option<HashMap<String, Bind>>);
    fn get_id(&self) -> &Option<String>;
}

#[cfg(feature = "ops")]
pub trait Wrap {
    type Target;
    fn wrap(self) -> Self::Target;
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
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

impl JsType {
    pub fn input_type(&self) -> &'static str {
        match self {
            Self::bool => "checkbox",
            Self::number => "number",
            Self::text => "text",
            Self::password => "password",
            Self::button => "button",
            Self::submit => "submit",
        }
    }

    pub fn default_value(&self) -> Value {
        match self {
            Self::number => to_value(0),
            Self::bool => to_value(false),
            _ => to_value(""),
        }
        .unwrap()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
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
        #[cfg(feature = "dioxus")]
        #[allow(dead_code)]
        #[serde(skip)]
        signal: Option<Signal<Value>>,
    },
    Submit {
        submit: bool,
        #[cfg(feature = "dioxus")]
        #[allow(dead_code)]
        #[serde(skip)]
        signal: Option<Signal<Value>>,
    },
    Default {},
}

impl Default for BindVariant {
    fn default() -> Self {
        BindVariant::Default {}
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct Bind {
    #[serde(flatten)]
    pub variant: BindVariant,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<JsType>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "dioxus", derive(Props))]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(any(feature = "ops", feature = "classify"), derive(ClassifyAttrs))]
pub struct ClassAttr {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub class: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "dioxus", derive(Props))]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(any(feature = "ops", feature = "classify"), derive(ClassifyAttrs))]
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
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub enum PosH {
    #[allow(non_camel_case_types)]
    left(String),
    #[allow(non_camel_case_types)]
    right(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub enum PosV {
    #[allow(non_camel_case_types)]
    top(String),
    #[allow(non_camel_case_types)]
    bottom(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "dioxus", derive(Props))]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(any(feature = "ops", feature = "classify"), derive(ClassifyAttrs))]
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
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub enum Direction {
    U,
    D,
    L,
    R,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "dioxus", derive(Props))]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(any(feature = "ops", feature = "classify"), derive(ClassifyAttrs))]
pub struct DirectionAttr {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub class: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<Direction>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "dioxus", derive(Props))]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(any(feature = "ops", feature = "classify"), derive(ClassifyAttrs))]
pub struct StyleAttr {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub class: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "dioxus", derive(Props))]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "ops", derive(BrickOps))]
#[cfg_attr(feature = "classify", derive(ClassifyBrick))]
pub struct Placeholder {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attrs: Option<ClassAttr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bind: Option<HashMap<String, Bind>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub: Option<Vec<Brick>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "dioxus", derive(Props))]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "ops", derive(BrickOps))]
#[cfg_attr(feature = "classify", derive(ClassifyBrick))]
pub struct Chart {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attrs: Option<ClassAttr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bind: Option<HashMap<String, Bind>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "dioxus", derive(Props))]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "ops", derive(BrickOps))]
#[cfg_attr(feature = "classify", derive(ClassifyBrick))]
pub struct Diagram {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attrs: Option<ClassAttr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bind: Option<HashMap<String, Bind>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "dioxus", derive(Props))]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "ops", derive(BrickOps))]
#[cfg_attr(feature = "classify", derive(ClassifyBrick))]
pub struct Float {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attrs: Option<PositionAttr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bind: Option<HashMap<String, Bind>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub: Option<Vec<Brick>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "dioxus", derive(Props))]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(any(feature = "ops", feature = "classify"), derive(ClassifyAttrs))]
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
#[cfg_attr(feature = "dioxus", derive(Props))]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "ops", derive(BrickOps))]
#[cfg_attr(feature = "classify", derive(ClassifyBrick))]
pub struct Fold {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attrs: Option<FoldAttr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bind: Option<HashMap<String, Bind>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub: Option<Vec<Brick>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item: Option<Vec<Brick>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "dioxus", derive(Props))]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(any(feature = "ops", feature = "classify"), derive(ClassifyAttrs))]
pub struct FormAttr {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub class: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instant: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "dioxus", derive(Props))]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "ops", derive(BrickOps))]
#[cfg_attr(feature = "classify", derive(ClassifyBrick))]
pub struct Form {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attrs: Option<FormAttr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bind: Option<HashMap<String, Bind>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub: Option<Vec<Brick>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "dioxus", derive(Props))]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "ops", derive(BrickOps))]
#[cfg_attr(feature = "classify", derive(ClassifyBrick))]
pub struct Popup {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attrs: Option<DirectionAttr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bind: Option<HashMap<String, Bind>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub: Option<Vec<Brick>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "dioxus", derive(Props))]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "ops", derive(BrickOps))]
#[cfg_attr(feature = "classify", derive(ClassifyBrick))]
pub struct Svg {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attrs: Option<SizeAttr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bind: Option<HashMap<String, Bind>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub: Option<Vec<Brick>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "dioxus", derive(Props))]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "ops", derive(BrickOps))]
#[cfg_attr(feature = "classify", derive(ClassifyBrick))]
pub struct Group {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attrs: Option<StyleAttr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bind: Option<HashMap<String, Bind>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub: Option<Vec<Brick>>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "dioxus", derive(Props))]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "ops", derive(BrickOps))]
#[cfg_attr(feature = "classify", derive(ClassifyBrick))]
pub struct Path {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attrs: Option<ClassAttr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bind: Option<HashMap<String, Bind>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "dioxus", derive(Props))]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(any(feature = "ops", feature = "classify"), derive(ClassifyAttrs))]
pub struct RackAttr {
    #[serde(default)]
    pub scroll: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub class: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "dioxus", derive(Props))]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "ops", derive(BrickOps))]
#[cfg_attr(feature = "classify", derive(ClassifyBrick))]
pub struct Rack {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attrs: Option<RackAttr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bind: Option<HashMap<String, Bind>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub: Option<Vec<Brick>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item: Option<Vec<Brick>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "dioxus", derive(Props))]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(any(feature = "ops", feature = "classify"), derive(ClassifyAttrs))]
pub struct ButtonAttr {
    #[serde(default)]
    pub oneshot: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub class: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "dioxus", derive(Props))]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "ops", derive(BrickOps))]
#[cfg_attr(feature = "classify", derive(ClassifyBrick))]
pub struct Button {
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attrs: Option<ButtonAttr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bind: Option<HashMap<String, Bind>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "dioxus", derive(Props))]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(any(feature = "ops", feature = "classify"), derive(ClassifyAttrs))]
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
#[cfg_attr(feature = "dioxus", derive(Props))]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "ops", derive(BrickOps))]
#[cfg_attr(feature = "classify", derive(ClassifyBrick))]
pub struct Image {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attrs: Option<ImageAttr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bind: Option<HashMap<String, Bind>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "dioxus", derive(Props))]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "ops", derive(BrickOps))]
#[cfg_attr(feature = "classify", derive(ClassifyBrick))]
pub struct Input {
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attrs: Option<ClassAttr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bind: Option<HashMap<String, Bind>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "dioxus", derive(Props))]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "ops", derive(BrickOps))]
#[cfg_attr(feature = "classify", derive(ClassifyBrick))]
pub struct Select {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attrs: Option<ClassAttr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bind: Option<HashMap<String, Bind>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub: Option<Vec<Brick>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "dioxus", derive(Props))]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "ops", derive(BrickOps))]
#[cfg_attr(feature = "classify", derive(ClassifyBrick))]
pub struct Table {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub: Option<Vec<Brick>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "dioxus", derive(Props))]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "ops", derive(BrickOps))]
#[cfg_attr(feature = "classify", derive(ClassifyBrick))]
pub struct Thead {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub: Option<Vec<Brick>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "dioxus", derive(Props))]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "ops", derive(BrickOps))]
#[cfg_attr(feature = "classify", derive(ClassifyBrick))]
pub struct Tbody {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub: Option<Vec<Brick>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "dioxus", derive(Props))]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "ops", derive(BrickOps))]
#[cfg_attr(feature = "classify", derive(ClassifyBrick))]
pub struct Tr {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub: Option<Vec<Brick>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "dioxus", derive(Props))]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "ops", derive(BrickOps))]
#[cfg_attr(feature = "classify", derive(ClassifyBrick))]
pub struct Th {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub: Option<Vec<Brick>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "dioxus", derive(Props))]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "ops", derive(BrickOps))]
#[cfg_attr(feature = "classify", derive(ClassifyBrick))]
pub struct Td {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub: Option<Vec<Brick>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "dioxus", derive(Props))]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(any(feature = "ops", feature = "classify"), derive(ClassifyAttrs))]
pub struct TextAttr {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub class: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "dioxus", derive(Props))]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "ops", derive(BrickOps))]
#[cfg_attr(feature = "classify", derive(ClassifyBrick))]
pub struct Text {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attrs: Option<TextAttr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bind: Option<HashMap<String, Bind>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "dioxus", derive(Props))]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "ops", derive(BrickOps))]
#[cfg_attr(feature = "classify", derive(ClassifyBrick))]
pub struct TextArea {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attrs: Option<ClassAttr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bind: Option<HashMap<String, Bind>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "dioxus", derive(Props))]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(any(feature = "ops", feature = "classify"), derive(ClassifyAttrs))]
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
#[cfg_attr(feature = "dioxus", derive(Props))]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "ops", derive(BrickOps))]
#[cfg_attr(feature = "classify", derive(ClassifyBrick))]
pub struct Case {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attrs: Option<CaseAttr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bind: Option<HashMap<String, Bind>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub: Option<Vec<Brick>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "dioxus", derive(Props))]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "ops", derive(BrickOps))]
#[cfg_attr(feature = "classify", derive(ClassifyBrick))]
pub struct Render {
    name: String,
    data: Map<String, Value>,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[cfg_attr(feature = "ops", derive(BrickOps))]
#[cfg_attr(feature = "classify", derive(ClassifyVariant))]
#[serde(tag = "type")]
pub enum Brick {
    case(Case),
    #[render_brick(has_id = "true")]
    placeholder(Placeholder),
    #[render_brick(has_id = "true")]
    chart(Chart),
    #[render_brick(has_id = "true")]
    diagram(Diagram),
    float(Float),
    #[render_brick(has_id = "true")]
    fold(Fold),
    form(Form),
    popup(Popup),
    svg(Svg),
    group(Group),
    path(Path),
    #[render_brick(has_id = "true")]
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
    #[cfg(feature = "render")]
    render(Render),
}

#[cfg(feature = "ops")]
impl Brick {
    pub fn cmp_id(&self, other: &Self) -> bool {
        let Some(id) = self.get_id() else {
            return false;
        };
        let Some(oid) = other.get_id() else {
            return false;
        };
        id == oid
    }
}

/*
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(tag = "type")]
pub enum JsonTableBrick {
    thead,
    tbody,
    tr,
    th,
    td,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(tag = "type")]
pub enum JsonSvgBrick {
    group,
    path,
}
*/
