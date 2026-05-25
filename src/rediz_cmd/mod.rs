use resp::{self, Value};
use std::net::TcpStream;

pub mod echo;

pub trait Cmd {
    fn execute(&self, argv: Vec<Value>, stream: &mut TcpStream) -> Result<(), std::io::Error>;
}
