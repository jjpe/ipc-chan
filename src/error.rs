//!

use serde_json::{Error as JsonError};
use zmq::{Error as ZmqError};


pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    /// I/O failure
    IoError(std::io::Error),
    /// JSON de/serialization failure
    JsonError(JsonError),
    /// Expected the bytes to be `UTF-8`, but they're not
    NotUtf8Error(Vec<u8>),
    /// Failed to deserialize from TOML file
    TomlDeserializeError(toml::de::Error),
    /// Failed to serialize to TOML file
    TomlSerializeError(toml::ser::Error),
    /// ZeroMQ failure
    ZmqError(ZmqError),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self { Self::IoError(e) }
}

impl From<JsonError> for Error {
    fn from(e: JsonError) -> Self { Self::JsonError(e) }
}

impl From<toml::de::Error> for Error {
    fn from(e: toml::de::Error) -> Self { Self::TomlDeserializeError(e) }
}

impl From<toml::ser::Error> for Error {
    fn from(e: toml::ser::Error) -> Self { Self::TomlSerializeError(e) }
}

impl From<ZmqError> for Error {
    fn from(e: ZmqError) -> Self { Self::ZmqError(e) }
}
