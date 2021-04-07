//! Toml config module

use crate::error::{Error, Result};
use serde_derive::{Deserialize, Serialize};
use std::env::current_dir;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Deserialize, Serialize)]
pub struct Config {
    pub host: String,
    pub port: usize,
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
        let toml_path = toml_path.as_ref();
        let toml_path = Self::find_in_ancestor_or_home_dir(&toml_path)
            .ok_or_else(|| Error::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("TOML file: {}", toml_path.display())
            )))?;

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

    /// Search for a file in ancestor directories, then in $HOME.
    /// First in the parent dir, then in the parent's parent dir, etc.
    /// Return `None` if the file could not be found anywhere.
    #[inline(always)]
    fn find_in_ancestor_or_home_dir<P: AsRef<Path>>(path: P) -> Option<PathBuf> {
        let path: PathBuf = path.as_ref().to_path_buf();
        if path.exists() { // File found in current directory
            return Some(path);
        }
        let exists = |p: &Option<PathBuf>| p.as_ref().map(|b| b.exists());
        let file_name: String = path.file_name()?.to_str()?.to_string();
        let mut buf: Option<PathBuf> = Some(path.clone());
        if Some(Path::new("")) == path.parent() { // No parent directory
            buf = current_dir().map(|cwd| cwd.join(&file_name)).ok();
        }
        while let Some(b) = buf.as_ref() { // NOTE Search ancestor directories
            let parent: PathBuf = b.parent().map(|p| p.to_path_buf())?;
            buf = match parent.parent() {
                Some(gp) if gp.exists() => Some(gp.join(&file_name)),
                _ => None,
            };
            if exists(&buf) == Some(true) { return buf; }
        }
        if buf.is_none() || Some(false) == exists(&buf) { // NOTE search $HOME
            if let Some(home_dir_path) = dirs::home_dir() {
                buf = Some(home_dir_path.join(&file_name));
            }
        }
        match buf {
            Some(b) if b.exists() => Some(b),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Deserialize, Serialize)]
pub enum OverwritePolicy {
    DontOverwrite,
    Overwrite,
}
