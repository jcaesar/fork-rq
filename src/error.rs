use csv;
use glob;
use protobuf;
use rmpv;
use serde_cbor;
use serde_hjson;
use serde_json;
use serde_protobuf;
use serde_yaml;
use std::io;
use std::string;
use thiserror::Error;
use toml;
use yaml_rust;

use std::result;

pub type Result<A> = result::Result<A, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("protobuf error")]
    Protobuf(#[from] serde_protobuf::error::Error),
    #[error("IO error")]
    Io(#[from] io::Error),
    #[error("UTF-8 error")]
    Utf8(#[from] string::FromUtf8Error),
    #[error("native protobuf error")]
    NativeProtobuf(#[from] protobuf::ProtobufError),
    #[error("MessagePack encode error")]
    MessagePackEncode(#[from] rmpv::encode::Error),
    #[error("Avro error")]
    Avro(#[from] avro_rs::DeError),
    #[error("CBOR error")]
    Cbor(#[from] serde_cbor::error::Error),
    #[error("HJSON error")]
    Hjson(#[from] serde_hjson::Error),
    #[error("JSON error")]
    Json(#[from] serde_json::Error),
    #[error("YAML error")]
    Yaml(#[from] serde_yaml::Error),
    #[error("YAML scan error")]
    YamlScan(#[from] yaml_rust::ScanError),
    #[error("TOML deserialize error")]
    TomlDeserialize(#[from] toml::de::Error),
    #[error("TOML serialize error")]
    TomlSerialize(#[from] toml::ser::Error),
    #[error("SMILE error")]
    Smile(#[from] serde_smile::Error),
    #[error("glob error")]
    Glob(#[from] glob::GlobError),
    #[error("glob pattern error")]
    GlobPattern(#[from] glob::PatternError),
    #[error("CSV error")]
    Csv(#[from] csv::Error),
    #[error("MessagePack decode error")]
    MessagePackDecode(#[from] rmpv::decode::Error),
    #[error("unimplemented: {}", msg)]
    Unimplemented { msg: String },
    #[error("illegal state: {}", msg)]
    IllegalState { msg: String },
    #[error("format error: {}", msg)]
    Format { msg: String },
    #[error("internal error: {}", _0)]
    Internal(&'static str),
    #[error("{}", _0)]
    Message(String),
}

impl Error {
    pub fn unimplemented(msg: String) -> Self {
        Self::Unimplemented { msg }
    }

    pub fn illegal_state(msg: String) -> Self {
        Self::IllegalState { msg }
    }
}
