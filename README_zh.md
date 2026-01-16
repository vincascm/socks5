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
- **支持多种异步运行时**: 通过 `futures-lite`，支持 `tokio` 及 `smol` 异步运行时。

## 如何使用

### 添加到依赖

在您的 `Cargo.toml` 文件中添加相应的 crate：

```toml
[dependencies]
# 如果你需要 SOCKS5 核心协议
socks5 = { git = "https://github.com/vincascm/socks5.git" }

# 如果你需要客户端功能
socks5-client = { git = "https://github.com/vincascm/socks5.git" }

# 如果你需要服务端功能
socks5-server = { git = "https://github.com/vincascm/socks5.git" }
```

### 示例

客户端参见 [examples](/examples/examples)
服务端参见 [socks5-server](https://github.com/vincascm/socks5-server)

## 贡献

欢迎任何形式的贡献，无论是提交 Issue 还是 Pull Request。

## 授权协议

本项目采用 [MIT](LICENSE-MIT) 或 [Apache-2.0](LICENSE-APACHE) 授权。
