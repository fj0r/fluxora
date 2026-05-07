use serde::{de::DeserializeOwned, Serialize};
use std::str::FromStr;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CodecError {
    #[error("Serialization failed: {0}")]
    Encode(String),
    #[error("Deserialization failed: {0}")]
    Decode(String),
    #[error("Unsupported codec type: {0}")]
    Unsupported(String),
}

/// 用于配置文件解析 (如 config.toml 中的 codec = "bincode")
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Deserialize)]
pub enum CodecType {
    Json,
    #[default]
    Bincode,
    Cbor,
}

impl FromStr for CodecType {
    type Err = CodecError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().trim() {
            "json" => Ok(Self::Json),
            "bincode" => Ok(Self::Bincode),
            "cbor" => Ok(Self::Cbor),
            _ => Err(CodecError::Unsupported(s.into())),
        }
    }
}

/// 用于运行时执行 (替代 Box<dyn Codec>)
/// 去除了冗余的内部结构体，直接内联逻辑
#[derive(Clone)]
pub enum ActiveCodec {
    Json,
    Bincode,
    Cbor,
}

impl ActiveCodec {
    pub fn new(t: CodecType) -> Self {
        match t {
            CodecType::Json => Self::Json,
            CodecType::Bincode => Self::Bincode,
            CodecType::Cbor => Self::Cbor,
        }
    }

    pub fn encode<T: Serialize>(&self, value: &T) -> Result<Vec<u8>, CodecError> {
        match self {
            Self::Json => serde_json::to_vec(value).map_err(|e| CodecError::Encode(e.to_string())),
            Self::Bincode => bincode::serialize(value).map_err(|e| CodecError::Encode(e.to_string())),
            Self::Cbor => {
                let mut out = Vec::new();
                ciborium::ser::into_writer(value, &mut out)
                    .map_err(|e| CodecError::Encode(e.to_string()))?;
                Ok(out)
            }
        }
    }

    pub fn decode<T: DeserializeOwned>(&self, bytes: &[u8]) -> Result<T, CodecError> {
        match self {
            Self::Json => serde_json::from_slice(bytes).map_err(|e| CodecError::Decode(e.to_string())),
            Self::Bincode => bincode::deserialize(bytes).map_err(|e| CodecError::Decode(e.to_string())),
            Self::Cbor => {
                let mut cursor = std::io::Cursor::new(bytes);
                ciborium::de::from_reader(&mut cursor).map_err(|e| CodecError::Decode(e.to_string()))
            }
        }
    }
}
