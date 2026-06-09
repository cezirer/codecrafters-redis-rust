use crate::db::Db;
use resp::{self, Value};
use std::{collections::HashMap, net::TcpStream, sync::Arc};

// 声明mod 文件名
pub mod echo;
pub mod get;
pub mod ping;
pub mod set;

pub trait Cmd {
    fn execute(
        &self,
        argv: Vec<Value>,
        stream: &mut TcpStream,
        db: &Db,
    ) -> Result<(), std::io::Error>;
}
