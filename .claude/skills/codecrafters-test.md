---
name: codecrafters-test
description: 根据用户所在的 CodeCrafters stage，自动生成测试用例
user-invocable: true
---

# CodeCrafters 测试用例生成器

根据用户当前所在的 CodeCrafters stage，分析现有代码实现，自动生成覆盖正常路径和边界情况的测试用例。

## 执行流程

### 第1步：确认 stage

如果 `$ARGUMENTS` 为空，询问用户当前所在 stage（编号或名称）。如果已提供，直接使用。

### 第2步：获取 stage 要求

两种途径并行获取：

1. **WebFetch** `https://app.codecrafters.io/challenges/redis/stages` — 提取对应 stage 的官方验收要求。
2. **WebSearch** `codecrafters redis <stage_name> stage requirements` — 获取社区讨论、补充细节。

将两者汇总，明确：
- 该 stage 要实现什么命令/功能
- 入参格式、返回值格式
- 边界条件和错误处理要求
- 如果有可选参数（如 SET 的 EX/PX），要列出

### 第3步：分析现有代码

读取项目中与该 stage 相关的所有源文件，至少包括：
- `src/main.rs` — 命令注册、dispatch 逻辑
- `src/rediz_cmd/mod.rs` — Cmd trait 定义
- `src/db.rs` — 数据存储层
- 对应命令的实现文件（如 `src/rediz_cmd/set.rs`, `get.rs` 等）

对每个文件，梳理：
- 参数校验逻辑（数量、类型）
- 每个 `match` / `if` 分支的执行路径
- 成功路径写什么响应
- 错误路径写什么响应
- 有没有未处理的 panic 风险（越界、unwrap）

### 第4步：编写测试用例

基于第2步的要求和第3步的代码分析，**直接在你的测试代码中补充**。测试结构如下：

```rust
#[cfg(test)]
mod tests {
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::time::Duration;

    fn send_cmd(stream: &mut TcpStream, cmd: &[&str]) -> String {
        let arr: Vec<resp::Value> = cmd.iter()
            .map(|s| resp::Value::Bulk(s.to_string()))
            .collect();
        stream.write_all(&resp::encode(&resp::Value::Array(arr))).unwrap();
        let mut buf = [0; 512];
        let n = stream.read(&mut buf).unwrap();
        String::from_utf8_lossy(&buf[..n]).to_string()
    }

    #[test]
    fn test_xxx() {
        let mut stream = TcpStream::connect("127.0.0.1:6379")
            .expect("请先 cargo run 启动服务");
        stream.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
        // ...
    }
}
```

测试用例必须覆盖：

| 类别 | 说明 | 示例 |
|------|------|------|
| **正常路径** | 命令正确执行 | SET key value → `+OK` |
| **参数不足** | argv 长度不够 | SET key（缺value）→ `-ERR` |
| **参数过多** | 多余参数看要求是否允许 | |
| **类型错误** | 非 Bulk 类型 | 不常见，但可测 |
| **边界值** | 空 key / 空 value / 超长字符串 | SET "" "" |
| **Key 不存在** | GET 不存在的 key | → `$-1` |
| **覆盖写** | SET 已存在的 key | → `+OK`，GET 返回新值 |
| **并发** | 多个连接同时操作 | 线程 spawn |

### 第5步：输出测试说明

在回复中列出：
- **stage 要求摘要**
- **发现了哪些代码路径**
- **写了哪些测试用例**（表格：用例名 + 覆盖场景 + 预期结果）
- **如何运行测试**（先 `cargo run`，再 `cargo test -- --nocapture`）
