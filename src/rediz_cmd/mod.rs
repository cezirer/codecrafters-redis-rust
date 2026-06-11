use crate::db::Db;
use resp::{self, Value};
use std::{collections::HashMap, net::TcpStream, sync::Arc};

// 声明mod 文件名
pub mod echo;
pub mod get;
pub mod ping;
pub mod set;
use crate::context::Context;
pub trait Cmd {
    fn execute<'a>(
        &self,
        argv: Vec<Value>,
        stream: &mut TcpStream,
        ctx: &Context<'a>,
    ) -> Result<(), std::io::Error>;
}
