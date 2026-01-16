# SOCKS5 协议实现

[English](/README.md)


这是一个使用 Rust 编写的，基于 `async` 的 SOCKS5 协议（RFC 1928）实现库。本项目是一个 Cargo 工作空间，包含以下几个核心部分。

## 项目结构

- `socks5`: 核心库，定义了 SOCKS5 协议所需的所有数据结构和序列化/反序列化逻辑，例如地址类型、请求/响应消息、认证方法等。
- `socks5-client`: 一个 SOCKS5 客户端库。它提供了一个 `connect` 函数，用于通过 SOCKS5 代理服务器连接到目标地址。
- `socks5-server`: 一个 SOCKS5 服务端库。它提供了一个 `proxy` 函数，用于处理传入的 SOCKS5 连接，当前支持 `CONNECT` 命令。

## 功能特性

- **完全异步**: 整个库基于 Rust 的 `async/await` 模型构建，适用于高性能网络应用。
- **模块化设计**: 通过 Cargo workspace 将核心协议、客户端和服务端逻辑清晰分离。
- **支持多种认证**: 客户端支持`无认证`和`用户名/密码`认证。服务端目前实现了`无认证`模式。
- **支持多种地址**: 支持 IPv4、IPv6 和域名地址类型。
- **符合 RFC 1928**: 严格遵循 SOCKS5 协议标准。

## 如何使用

### 添加到依赖

在您的 `Cargo.toml` 文件中添加相应的 crate：

```toml
[dependencies]
# 如果你需要 SOCKS5 核心协议
socks5 = { git = "https://github.com/vincascm/socks5.git", branch = "main" }

# 如果你需要客户端功能
socks5-client = { git = "https://github.com/vincascm/socks5.git", branch = "main" }

# 如果你需要服务端功能
socks5-server = { git = "https://github.com/vincascm/socks5.git", branch = "main" }
```
*请将 `your-repo` 替换为您的仓库地址。*

### 客户端示例

下面是一个使用 `socks5-client` 连接到 `example.com:80` 的例子。

```rust
use async_io::Async;
use futures_lite::io::Cursor;
use socks5::address::Address;
use socks5_client::{connect, Result};
use std::net::TcpStream;

async fn client_example() -> Result<()> {
    // 创建一个内存中的 stream 用于演示
    let mut stream = Cursor::new(Vec::new());

    // 目标地址
    let dest_addr = Address::from(( "example.com", 80));

    // 执行连接，这里不使用认证
    connect(&mut stream, dest_addr, None).await?;

    println!("Successfully connected via SOCKS5 proxy!");
    Ok(())
}
```

### 服务端示例

下面是一个使用 `socks5-server` 启动一个简单代理服务器的例子。

```rust
use async_io::Async;
use futures_lite::io::Cursor;
use socks5_server::{proxy, Result};
use std::net::{SocketAddr, Ipv4Addr};

async fn server_example() -> Result<()> {
    // 模拟一个客户端连接
    let mut stream = Cursor::new(Vec::new());

    // 模拟客户端地址
    let src_addr = SocketAddr::new(Ipv4Addr::new(127, 0, 0, 1).into(), 8080);

    // 运行代理逻辑
    // 注意：实际应用中 stream 应该是来自网络的真实 TCP 连接
    proxy(&mut stream, src_addr).await?;

    println!("Proxy logic completed!");
    Ok(())
}

```

## 贡献

欢迎任何形式的贡献，无论是提交 Issue 还是 Pull Request。

## 授权协议

本项目采用 [MIT](LICENSE-MIT) 或 [Apache-2.0](LICENSE-APACHE) 授权。
