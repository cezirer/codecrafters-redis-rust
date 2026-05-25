#![allow(unused_imports)]
use resp::{Decoder, Value, encode};
use std::collections::HashMap;
use std::io::{BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::Arc;
use std::thread;
mod parser;
mod rediz_cmd;
use rediz_cmd::Cmd;
pub struct CmdRegistry {
    handlers: HashMap<String, Box<dyn Cmd + Send + Sync>>,
}
impl CmdRegistry {
    pub fn new() -> Self {
        let mut handlers = HashMap::new();
        handlers.insert(
            "echo".to_string(),
            Box::new(rediz_cmd::echo::EchoCmd) as Box<dyn Cmd + Send + Sync>,
        );
        handlers.insert(
            "ping".to_string(),
            Box::new(rediz_cmd::ping::PingCmd) as Box<dyn Cmd + Send + Sync>,
        );
        Self { handlers: handlers }
    }
    pub fn get(&self, cmd_name: &str) -> Option<&Box<dyn Cmd + Send + Sync>> {
        self.handlers.get(cmd_name)
    }
}
fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");
    // Uncomment the code below to pass the first stage
    //
    let cmd_registry = CmdRegistry::new();
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    let shared_registry = Arc::new(cmd_registry);
    for stream in listener.incoming() {
        let registry_clone = Arc::clone(&shared_registry);
        thread::spawn(move || match stream {
            Ok(mut stream) => {
                let mut write_stream = stream.try_clone().unwrap();
                let mut reader = BufReader::new(stream);
                let mut decoder = Decoder::new(reader);
                loop {
                    match decoder.decode() {
                        Ok(value) => match value {
                            Value::Array(a) => {
                                if a.len() == 0 {
                                    eprintln!("the len is 0");
                                    break;
                                }

                                let cmd_name = if let Value::Bulk(b) = &a[0] {
                                    b.clone().to_lowercase()
                                } else {
                                    eprintln!("");
                                    break;
                                };
                                // println!("{:?}", cmd_name);
                                if let Some(cmd) = registry_clone.get(&cmd_name) {
                                    let argv: Vec<Value> = if a.len() == 1 {
                                        vec![]
                                    } else {
                                        a[1..].to_vec()
                                    };
                                    cmd.execute(argv, &mut write_stream);
                                } else {
                                    eprintln!("")
                                }
                            }
                            _ => {
                                eprintln!("error");
                                break;
                            }
                        },
                        Err(e) => {
                            eprintln!("{}", e);
                            break;
                        }
                    };
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        });
        // handle.join().unwrap();
    }
}

#[cfg(test)]
mod tests {
    use resp::{self, Value, encode};
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::time::Duration;
    #[test]
    fn test_ping_pong() {
        // 注意：运行此测试时，确保主程序 cargo run 已经在后台启动了
        let mut stream =
            TcpStream::connect("127.0.0.1:6379").expect("无法连接到服务器，请确保主程序已在运行！");

        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        // 构造一个符合 Redis 规范的数组命令：["PING"]
        let cmd = Value::Array(vec![
            Value::Bulk(String::from("ECHO")),
            Value::Bulk(String::from("hey")),
        ]);

        // 变成字节数组发出去
        stream.write_all(&encode(&cmd)).unwrap();

        // // 读取服务器返回的响应
        let mut buf = [0; 512];
        let bytes_read = stream.read(&mut buf).unwrap();
        let response = String::from_utf8_lossy(&buf[..bytes_read]);

        // 断言返回值是否为 +PONG\r\n
        // assert_eq!(response, "+PONG\r\n");
        println!("{}", response);
        // println!("测试通过！收到了预期的 PONG");
    }
}
