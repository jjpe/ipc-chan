//! Toml config module

use crate::error::Result;
use serde_derive::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Deserialize, Serialize)]
pub struct Config {
    pub(crate) host: String,
    pub(crate) port: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 10001,
        }
    }
}

impl Config {
    /// Parse a TOML config file.
    /// If the file can't be found, default settings are assumed and returned.
    pub fn parse_toml<P: AsRef<Path>>(toml_path: P) -> Result<Self> {
        if let Ok(mut file) = File::open(toml_path) {
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            return Ok(toml::from_str(&contents)?);
        } else {
            Ok(Self::default())
        }
    }

    /// Write `self` to a TOML config file @ `toml_path`.
    /// If `overwrite_policy` is `OverwritePolicy::Overwrite`, the file will
    /// be written regardless of whether or not it previously existed.
    /// However, if `overwrite_policy` is `OverwritePolicy::DontOverwrite`, the
    /// file will only be written iff. it did not previously exist.
    pub fn write_toml<P: AsRef<Path>>(
        &self,
        toml_path: P,
        overwrite_policy: OverwritePolicy
    ) -> Result<()> {
        let toml_path: &Path = toml_path.as_ref();
        let mut file = File::create(toml_path)?;
        match overwrite_policy {
            OverwritePolicy::DontOverwrite if toml_path.exists() => {
                // NOP  // NOTE don't remove this branch
            },
            OverwritePolicy::DontOverwrite | OverwritePolicy::Overwrite => {
                let contents: String = toml::to_string_pretty(&self)?;
                file.write_all(contents.as_bytes())?;
            },
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Deserialize, Serialize)]
pub enum OverwritePolicy {
    DontOverwrite,
    Overwrite,
}
