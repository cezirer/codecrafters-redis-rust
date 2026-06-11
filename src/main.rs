#![allow(unused_imports)]
use resp::{Decoder, Value, encode};
use std::collections::HashMap;
use std::io::{BufReader, Read, Write};
use std::net::TcpListener;
use std::sync::Arc;
use std::thread;
mod context;
mod db;
mod rediz_cmd;
use context::Context;
use db::Db;

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
        handlers.insert(
            "set".to_string(),
            Box::new(rediz_cmd::set::SetCmd) as Box<dyn Cmd + Send + Sync>,
        );
        handlers.insert(
            "get".to_string(),
            Box::new(rediz_cmd::get::GetCmd) as Box<dyn Cmd + Send + Sync>,
        );
        Self { handlers: handlers }
    }
    pub fn get(&self, cmd_name: &str) -> Option<&Box<dyn Cmd + Send + Sync>> {
        self.handlers.get(cmd_name)
    }
}
fn main() {
    println!("Logs from your program will appear here!");
    let cmd_registry = CmdRegistry::new();
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    let shared_registry = Arc::new(cmd_registry);
    let db = Arc::new(Db::new());
    for stream in listener.incoming() {
        let registry_clone = Arc::clone(&shared_registry);
        let db_clone = Arc::clone(&db);
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
                                    let ctx = Context { db: &db_clone };
                                    let _ = cmd.execute(argv, &mut write_stream, &ctx);
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
    use std::thread::sleep;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    /// 发送 RESP 命令并读取响应
    fn send_cmd(stream: &mut TcpStream, cmd: &[&str]) -> String {
        let arr: Vec<Value> = cmd.iter().map(|s| Value::Bulk(s.to_string())).collect();
        stream.write_all(&encode(&Value::Array(arr))).unwrap();
        let mut buf = [0; 512];
        let n = stream.read(&mut buf).unwrap();
        String::from_utf8_lossy(&buf[..n]).to_string()
    }

    /// 创建一个连接到本地 Redis 服务器的 stream
    fn connect() -> TcpStream {
        let mut stream =
            TcpStream::connect("127.0.0.1:6379").expect("无法连接到服务器，请先 cargo run 启动！");
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        stream
    }

    fn unique_key(name: &str) -> String {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        format!("{name}:{nanos}")
    }

    // ─── SET 正常路径 ───

    #[test]
    fn test_set_ok() {
        let mut stream = connect();
        let resp = send_cmd(&mut stream, &["SET", "apple", "red"]);
        assert_eq!(resp, "+OK\r\n");
    }

    #[test]
    fn test_set_empty_key() {
        let mut stream = connect();
        let resp = send_cmd(&mut stream, &["SET", "", "value"]);
        // 空 key 是合法的，应该返回 +OK
        assert_eq!(resp, "+OK\r\n");
    }

    #[test]
    fn test_set_empty_value() {
        let mut stream = connect();
        let resp = send_cmd(&mut stream, &["SET", "mykey", ""]);
        // 空 value 也是合法的，应该返回 +OK
        assert_eq!(resp, "+OK\r\n");
    }

    // ─── SET 错误路径 ───

    #[test]
    fn test_set_wrong_args_count() {
        let mut stream = connect();
        let resp = send_cmd(&mut stream, &["SET", "onlykey"]);
        assert!(resp.starts_with("-ERR"), "期望 -ERR，实际收到: {resp}");
    }

    // ─── GET 正常路径 ───

    #[test]
    fn test_get_existing_key() {
        let mut stream = connect();
        // 先 SET
        send_cmd(&mut stream, &["SET", "banana", "yellow"]);
        // 再 GET（同一个连接）
        let resp = send_cmd(&mut stream, &["GET", "banana"]);
        assert_eq!(resp, "$6\r\nyellow\r\n");
    }

    #[test]
    fn test_get_non_existing_key() {
        let mut stream = connect();
        let resp = send_cmd(&mut stream, &["GET", "nonexistent"]);
        assert_eq!(resp, "$-1\r\n");
    }

    #[test]
    fn test_get_after_overwrite() {
        let mut stream = connect();
        send_cmd(&mut stream, &["SET", "color", "red"]);
        send_cmd(&mut stream, &["SET", "color", "blue"]);
        let resp = send_cmd(&mut stream, &["GET", "color"]);
        assert_eq!(resp, "$4\r\nblue\r\n");
    }

    #[test]
    fn test_get_empty_value() {
        let mut stream = connect();
        send_cmd(&mut stream, &["SET", "emptykey", ""]);
        let resp = send_cmd(&mut stream, &["GET", "emptykey"]);
        // 空字符串 → $0\r\n\r\n
        assert_eq!(resp, "$0\r\n\r\n");
    }

    // ─── GET 错误路径 ───

    #[test]
    fn test_get_wrong_args_count() {
        let mut stream = connect();
        // GET 不带参数
        let resp = send_cmd(&mut stream, &["GET"]);
        assert!(resp.starts_with("-ERR"), "期望 -ERR，实际收到: {resp}");
    }

    // ─── 序列化/反序列化 round-trip ───

    #[test]
    fn test_set_get_roundtrip() {
        let mut stream = connect();
        let cases = vec![
            ("hello", "world"),
            ("number", "42"),
            ("special", "!@#$%"),
            ("中文", "测试"),
        ];
        for (k, v) in &cases {
            let resp = send_cmd(&mut stream, &["SET", k, v]);
            assert_eq!(resp, "+OK\r\n", "SET {k} {v} 失败");
            let resp = send_cmd(&mut stream, &["GET", k]);
            let expected = format!("${}\r\n{}\r\n", v.len(), v);
            assert_eq!(resp, expected, "GET {k} 返回值不匹配");
        }
    }

    // ─── SET PX 过期时间 ───

    #[test]
    fn test_set_px_value_is_available_before_expiry() {
        let mut stream = connect();
        let key = unique_key("px-before-expiry");

        let resp = send_cmd(&mut stream, &["SET", &key, "value", "PX", "500"]);
        assert_eq!(resp, "+OK\r\n");

        let resp = send_cmd(&mut stream, &["GET", &key]);
        assert_eq!(resp, "$5\r\nvalue\r\n");
    }

    #[test]
    fn test_set_px_key_expires_after_duration() {
        let mut stream = connect();
        let key = unique_key("px-after-expiry");

        let resp = send_cmd(&mut stream, &["SET", &key, "gone", "PX", "100"]);
        assert_eq!(resp, "+OK\r\n");

        sleep(Duration::from_millis(150));

        let resp = send_cmd(&mut stream, &["GET", &key]);
        assert_eq!(resp, "$-1\r\n");
    }

    #[test]
    fn test_set_without_px_overwrites_and_clears_existing_expiry() {
        let mut stream = connect();
        let key = unique_key("px-overwrite");

        let resp = send_cmd(&mut stream, &["SET", &key, "short", "PX", "100"]);
        assert_eq!(resp, "+OK\r\n");

        let resp = send_cmd(&mut stream, &["SET", &key, "persist"]);
        assert_eq!(resp, "+OK\r\n");

        sleep(Duration::from_millis(150));

        let resp = send_cmd(&mut stream, &["GET", &key]);
        assert_eq!(resp, "$7\r\npersist\r\n");
    }
}
