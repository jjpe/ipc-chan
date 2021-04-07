//!

mod error;
mod config;

pub use crate::error::{Error, Result};
pub use crate::config::Config;
use std::path::Path;
use serde::{Deserialize, Serialize};


/// It's a little passive-aggressive, but it'll work.
const ACK: &str = "K";

/// A convenience macro to "print" formatted text to a `Source`.
#[macro_export]
macro_rules! sendstr {
    ($source:expr, $fmt:expr $(, $arg:expr)*) => {{
        let source: &mut $crate::Source = &mut $source;
        source.send(&format!($fmt $(, $arg)*))
    }};
}


pub struct Source {
    #[allow(unused)]
    ctx: zmq::Context,
    socket: zmq::Socket,
    #[allow(unused)]
    cfg: Config,
}

impl Source {
    pub fn from_toml<P: AsRef<Path>>(toml_path: P) -> Result<Self> {
        let cfg = Config::parse_toml(toml_path)?;
        Self::from_config(cfg)
    }

    pub fn from_config(cfg: Config) -> Result<Self> {
        let ctx = zmq::Context::new();
        let socket = ctx.socket(zmq::REQ)?;
        socket.connect(&format!("tcp://{}:{}", cfg.host, cfg.port))?;
        Ok(Self { ctx, socket, cfg })
    }

    /// Send a value of type `V`.
    /// Return `Ok(())` if the value was sent successfully;
    /// Otherwise return an error.
    pub fn send<V>(&mut self, value: &V) -> Result<()>
    where V: ?Sized + Serialize {
        imp::send(&mut self.socket, value)?;
        let reply: String = imp::recv(&mut self.socket)?;
        debug_assert_eq!(reply, ACK);
        Ok(())
    }

    #[inline(always)]
    pub fn config(&self) -> &Config { &self.cfg }
}


pub struct Sink {
    #[allow(unused)]
    ctx: zmq::Context,
    socket: zmq::Socket,
    #[allow(unused)]
    cfg: Config,
}

impl Sink {
    pub fn from_toml<P: AsRef<Path>>(toml_path: P) -> Result<Self> {
        let cfg = Config::parse_toml(toml_path)?;
        Self::from_config(cfg)
    }

    pub fn from_config(cfg: Config) -> Result<Self> {
        let ctx = zmq::Context::new();
        let socket = ctx.socket(zmq::REP)?;
        socket.bind(&format!("tcp://*:{}", cfg.port))?;
        Ok(Self { ctx, socket, cfg })
    }


    pub fn recv<V>(&mut self) -> Result<V>
    where V: for<'de> Deserialize<'de> {
        let msg: V = imp::recv(&mut self.socket)?;
        imp::send(&mut self.socket, ACK)?;
        Ok(msg)
    }

    #[inline(always)]
    pub fn config(&self) -> &Config { &self.cfg }
}


mod imp {
    use super::*;

    const NO_FLAGS: i32 = 0;

    #[inline(always)]
    pub(super) fn send<V>(socket: &mut zmq::Socket, value: &V) -> Result<()>
    where V: ?Sized + Serialize {
        let s: String = serde_json::to_string(value)?;
        socket.send(&s, NO_FLAGS)?;
        Ok(())
    }

    #[inline(always)]
    pub(super) fn recv<V>(socket: &mut zmq::Socket) -> Result<V>
    where V: for<'de> Deserialize<'de> {
        match socket.recv_string(NO_FLAGS)? {
            Ok(s) => Ok(serde_json::from_str::<V>(&s)?),
            Err(bytes) => Err(Error::NotUtf8Error(bytes)),
        }
    }
}



#[cfg(test)]
mod tests {
    use crate::error::Result;
    use serde_derive::{Deserialize, Serialize};
    use super::*;

    #[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
    struct Foo(String, usize);

    #[test]
    fn send_and_receive_msg() -> Result<()> {
        let cfg = Config {
            host: "127.0.0.1".to_string(),
            port: 11001, // test-specific port
        };
        let mut source = Source::from_config(cfg.clone())?;
        let mut   sink =   Sink::from_config(cfg.clone())?;
        let thread_guard = std::thread::spawn(move || {
            let msg0: String = sink.recv().expect("Sink failed to receive MSG0");
            assert_eq!(msg0, "Hello World! 0");
            let msg1: String = sink.recv().expect("Sink failed to receive MSG1");
            assert_eq!(msg1, "Hello World! 1");
            let msg2: Foo = sink.recv().expect("Sink failed to receive MSG2");
            assert_eq!(msg2, Foo("Hello World! 2".to_string(), 42));
        });
        source.send("Hello World! 0")?;
        source.send("Hello World! 1")?;
        source.send(&Foo("Hello World! 2".to_string(), 42))?;
        thread_guard.join().unwrap();
        Ok(())
    }

    #[test]
    fn multiple_senders() -> Result<()> {
        let cfg = Config {
            host: "127.0.0.1".to_string(),
            port: 11002, // test-specific port
        };
        let mut source0 = Source::from_config(cfg.clone())?;
        let mut source1 = Source::from_config(cfg.clone())?;
        let mut    sink =   Sink::from_config(cfg.clone())?;
        let thread_guard = std::thread::spawn(move || {
            let msg0: String = sink.recv().expect("Sink failed to receive MSG0");
            assert_eq!(msg0, "Hello World! 0");
            let msg1: String = sink.recv().expect("Sink failed to receive MSG1");
            assert_eq!(msg1, "Hello World! 1");
            let msg2: Foo = sink.recv().expect("Sink failed to receive MSG2");
            assert_eq!(msg2, Foo("Hello World! 2".to_string(), 42));
        });
        source0.send("Hello World! 0")?;
        source1.send("Hello World! 1")?;
        source0.send(&Foo("Hello World! 2".to_string(), 42))?;
        thread_guard.join().unwrap();
        Ok(())
    }

    #[test]
    fn send_str_macro() -> Result<()> {
        let cfg = Config {
            host: "127.0.0.1".to_string(),
            port: 11003, // test-specific port
        };
        let mut source0 = Source::from_config(cfg.clone())?;
        let mut source1 = Source::from_config(cfg.clone())?;
        let mut    sink =   Sink::from_config(cfg.clone())?;
        let thread_guard = std::thread::spawn(move || {
            let msg0: String = sink.recv().expect("Sink failed to receive msg0");
            assert_eq!(msg0, "Hello World! 0");
            let msg1: String = sink.recv().expect("Sink failed to receive msg1");
            assert_eq!(msg1, "Hello World! 1");
            let msg2: Foo = sink.recv().expect("Sink failed to receive msg2");
            assert_eq!(msg2, Foo("Hello World! 2".to_string(), 42));
        });
        sendstr!(source0, "Hello World! {}", 0)?;
        sendstr!(source1, "Hello World! {}", 1)?;
        source0.send(&Foo("Hello World! 2".to_string(), 42))?;
        thread_guard.join().unwrap();
        Ok(())
    }

    #[test]
    fn read_config_file() -> Result<()> {
        let cfg = Config::parse_toml("ipc-chan.toml")?;
        let default_cfg = Config::default();
        assert_eq!(cfg.host, default_cfg.host);
        assert_eq!(cfg.port, default_cfg.port);
        Ok(())
    }

}
