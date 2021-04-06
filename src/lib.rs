//!

mod error;
mod config;

pub use crate::error::{Error, Result};
pub use crate::config::Config;
use std::path::Path;
use serde::{Deserialize, Serialize};


pub struct Ctx {
    ctx: zmq::Context,
    cfg: Config,
}

impl Ctx {
    pub fn new<P: AsRef<Path>>(toml_path: P) -> Result<Self> {
        Ok(Self {
            ctx: zmq::Context::new(),
            cfg: Config::parse_toml(toml_path)?,
        })
    }

    pub fn with_config(config: Config) -> Self {
        Self {
            ctx: zmq::Context::new(),
            cfg: config,
        }
    }

    pub fn channel(&self) -> Result<(Source, Sink)> {
        let   sink =   self.sink()?; // NOTE: first initialize the sink ...
        let source = self.source()?; // NOTE: ... and only then the source
        Ok((source, sink))
    }

    fn source(&self) -> Result<Source> {
        let source = Source(self.ctx.socket(zmq::REQ)?);
        source.0.connect(&format!("tcp://{}:{}", self.cfg.host, self.cfg.port))?;
        Ok(source)
    }

    fn sink(&self) -> Result<Sink> {
        let sink = Sink(self.ctx.socket(zmq::REP)?);
        sink.0.bind(&format!("tcp://*:{}", self.cfg.port))?;
        Ok(sink)
    }
}


/// It's a little passive-aggressive, but it'll work.
const ACK: &str = "K";


pub struct Source(zmq::Socket);

impl Source {
    /// Send a value of type `V`.
    /// Return `Ok(())` if the value was sent successfully;
    /// Otherwise return an error.
    pub fn send<V>(&mut self, value: &V) -> Result<()>
    where V: ?Sized + Serialize {
        imp::send(&mut self.0, value)?;
        let reply: String = imp::recv(&mut self.0)?;
        debug_assert_eq!(reply, ACK);
        Ok(())
    }
}


pub struct Sink(zmq::Socket);

impl Sink {
    pub fn recv<V>(&mut self) -> Result<V>
    where V: for<'de> Deserialize<'de> {
        let msg: V = imp::recv(&mut self.0)?;
        imp::send(&mut self.0, ACK)?;
        Ok(msg)
    }
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
        let ctx = Ctx::with_config(Config {
            host: "127.0.0.1".to_string(),
            port: 11001, // test-specific port
        });
        let (mut source, mut sink): (Source, Sink) = ctx.channel()?;
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
        let ctx = Ctx::with_config(Config {
            host: "127.0.0.1".to_string(),
            port: 11002, // test-specific port
        });
        let (mut source0, mut sink): (Source, Sink) = ctx.channel()?;
        let mut source1 = ctx.source()?;
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
    fn read_config_file() -> Result<()> {
        let ctx = Ctx::new("ipc-channel.toml")?;
        let cfg = Config::default();
        assert_eq!(ctx.cfg.host, cfg.host);
        assert_eq!(ctx.cfg.port, cfg.port);
        Ok(())
    }

}
