use resp::{self, Value};
use std::net::TcpStream;

// 声明mod 文件名
pub mod echo;
pub mod ping;
pub trait Cmd {
    fn execute(&self, argv: Vec<Value>, stream: &mut TcpStream) -> Result<(), std::io::Error>;
}
