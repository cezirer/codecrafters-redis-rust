use crate::context::Context;
use crate::db::Db;
use crate::rediz_cmd::Cmd;
use resp::{Value, encode};
use std::{io::Write, net::TcpStream};
pub struct EchoCmd;
impl Cmd for EchoCmd {
    fn execute(
        &self,
        argv: Vec<Value>,
        stream: &mut TcpStream,
        _: &Context,
    ) -> Result<(), std::io::Error> {
        let mut s = String::new();
        argv.iter().for_each(|v| {
            if let Value::Bulk(bulk_string) = v {
                s.push_str(bulk_string);
            }
        });
        // println!("{}", s);
        let mut bulk_s = encode(&Value::Bulk(s));
        stream.write_all(bulk_s.as_slice())?;
        Ok(())
    }
}
