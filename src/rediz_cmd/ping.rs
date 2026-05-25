// 声明trait 放在mod.rs里
use crate::rediz_cmd::Cmd;
use resp::{Value, encode};
use std::{io::Write, net::TcpStream};
pub struct PingCmd;

impl Cmd for PingCmd {
    fn execute(&self, argv: Vec<Value>, stream: &mut TcpStream) -> Result<(), std::io::Error> {
        let bulk_s = encode(&Value::Bulk("PONG".to_string()));
        stream.write_all(&bulk_s.as_slice())?;
        Ok(())
    }
}
