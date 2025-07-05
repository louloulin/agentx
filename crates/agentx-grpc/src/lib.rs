//! AgentX gRPC插件系统
//!
//! 提供基于gRPC的插件架构，支持A2A协议的分布式Agent通信

pub mod error;
pub mod converter;
pub mod plugin_bridge;
pub mod plugin_manager;
pub mod grpc_server;

// 生成的protobuf代码
pub mod proto {
    tonic::include_proto!("agentx.plugin.v1");
}

// 重新导出主要类型
pub use error::*;
pub use converter::*;
pub use plugin_bridge::*;
pub use plugin_manager::*;
pub use grpc_server::*;

/// gRPC插件系统版本
pub const GRPC_VERSION: &str = "0.1.0";

/// 默认gRPC端口
pub const DEFAULT_GRPC_PORT: u16 = 50051;

/// 默认插件注册表端口
pub const DEFAULT_REGISTRY_PORT: u16 = 50052;
