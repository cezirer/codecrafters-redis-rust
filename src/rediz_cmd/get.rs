use std::io::Write;

use resp::{Value, encode};

use crate::rediz_cmd::{Cmd, Context};
pub struct GetCmd;

impl Cmd for GetCmd {
    fn execute(
        &self,
        argv: Vec<resp::Value>,
        stream: &mut std::net::TcpStream,
        ctx: &Context,
    ) -> Result<(), std::io::Error> {
        let guard = ctx.db.kv_store.read().unwrap();
        if argv.len() < 1 {
            stream.write_all(b"-ERR wrong number of arguments for 'get' command\r\n")?;
            return Ok(());
        }
        let k = argv[0].clone();
        match k {
            Value::Bulk(k) => {
                if let Some(v) = guard.get(&k) {
                    let s = String::from_utf8_lossy(v).to_string();
                    let bulk_s = encode(&Value::Bulk(s));
                    stream.write_all(&bulk_s)?;
                } else {
                    stream.write_all(b"$-1\r\n")?;
                    return Ok(());
                }
            }
            _ => {
                stream.write_all(b"-ERR invalid key type\r\n")?;
            }
        }
        Ok(())
    }
}
