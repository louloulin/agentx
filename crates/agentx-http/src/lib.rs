//! AgentX HTTP/REST API Server
//! 
//! 基于Axum的HTTP服务器，提供RESTful API接口来访问A2A协议功能。
//! 支持OpenAPI文档生成、身份验证、CORS等企业级功能。

pub mod server;
pub mod handlers;
pub mod middleware;
pub mod models;
pub mod error;
pub mod config;
pub mod docs;
pub mod response;

pub use server::*;
pub use handlers::*;
pub use middleware::*;
pub use models::*;
pub use error::*;
pub use config::*;
pub use docs::*;
pub use response::*;

/// HTTP API版本
pub const API_VERSION: &str = "v1";

/// 默认服务器端口
pub const DEFAULT_PORT: u16 = 8080;
