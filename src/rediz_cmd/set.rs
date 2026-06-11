use resp::Value;

use crate::db::Db;
use crate::rediz_cmd::Cmd;
use crate::rediz_cmd::Context;
use std::io::{Error, Write};
use std::net::TcpStream;
pub struct SetCmd;

impl Cmd for SetCmd {
    fn execute(
        &self,
        argv: Vec<resp::Value>,
        stream: &mut TcpStream,
        ctx: &Context,
    ) -> Result<(), std::io::Error> {
        let mut guard = ctx.db.kv_store.write().unwrap();
        if argv.len() < 2 {
            stream.write_all(b"-ERR wrong number of arguments for 'set' command\r\n")?;
            return Ok(());
        }
        let (k, v) = (argv[0].clone(), argv[1].clone());
        match (k, v) {
            (Value::Bulk(k_string), Value::Bulk(v_string)) => {
                guard.insert(k_string, v_string.into_bytes());
                stream.write_all(b"+OK\r\n")?;
            }
            _ => stream.write_all(b"-ERR invalid key type\r\n")?,
        }
        Ok(())
    }
}
