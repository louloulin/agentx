//! gRPC插件系统错误类型定义

use thiserror::Error;
use tonic::{Code, Status};

/// gRPC插件系统错误类型
#[derive(Error, Debug)]
pub enum GrpcError {
    #[error("gRPC传输错误: {0}")]
    Transport(#[from] tonic::transport::Error),
    
    #[error("gRPC状态错误: {0}")]
    Status(#[from] tonic::Status),
    
    #[error("插件注册失败: {0}")]
    PluginRegistration(String),
    
    #[error("插件未找到: {plugin_id}")]
    PluginNotFound { plugin_id: String },
    
    #[error("Agent注册失败: {0}")]
    AgentRegistration(String),
    
    #[error("Agent未找到: {agent_id}")]
    AgentNotFound { agent_id: String },
    
    #[error("消息路由失败: {0}")]
    MessageRouting(String),
    
    #[error("协议转换错误: {0}")]
    ProtocolConversion(String),
    
    #[error("配置错误: {0}")]
    Configuration(String),
    
    #[error("序列化错误: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("A2A协议错误: {0}")]
    A2AProtocol(#[from] agentx_a2a::A2AError),
    
    #[error("内部错误: {0}")]
    Internal(String),
}

/// gRPC结果类型
pub type GrpcResult<T> = Result<T, GrpcError>;

impl From<GrpcError> for tonic::Status {
    fn from(err: GrpcError) -> Self {
        match err {
            GrpcError::Transport(_) => Status::new(Code::Unavailable, err.to_string()),
            GrpcError::Status(status) => status,
            GrpcError::PluginRegistration(_) => Status::new(Code::InvalidArgument, err.to_string()),
            GrpcError::PluginNotFound { .. } => Status::new(Code::NotFound, err.to_string()),
            GrpcError::AgentRegistration(_) => Status::new(Code::InvalidArgument, err.to_string()),
            GrpcError::AgentNotFound { .. } => Status::new(Code::NotFound, err.to_string()),
            GrpcError::MessageRouting(_) => Status::new(Code::Internal, err.to_string()),
            GrpcError::ProtocolConversion(_) => Status::new(Code::InvalidArgument, err.to_string()),
            GrpcError::Configuration(_) => Status::new(Code::FailedPrecondition, err.to_string()),
            GrpcError::Serialization(_) => Status::new(Code::InvalidArgument, err.to_string()),
            GrpcError::A2AProtocol(_) => Status::new(Code::Internal, err.to_string()),
            GrpcError::Internal(_) => Status::new(Code::Internal, err.to_string()),
        }
    }
}

impl GrpcError {
    /// 创建插件注册错误
    pub fn plugin_registration<S: Into<String>>(msg: S) -> Self {
        Self::PluginRegistration(msg.into())
    }
    
    /// 创建插件未找到错误
    pub fn plugin_not_found<S: Into<String>>(plugin_id: S) -> Self {
        Self::PluginNotFound {
            plugin_id: plugin_id.into(),
        }
    }
    
    /// 创建Agent注册错误
    pub fn agent_registration<S: Into<String>>(msg: S) -> Self {
        Self::AgentRegistration(msg.into())
    }
    
    /// 创建Agent未找到错误
    pub fn agent_not_found<S: Into<String>>(agent_id: S) -> Self {
        Self::AgentNotFound {
            agent_id: agent_id.into(),
        }
    }
    
    /// 创建消息路由错误
    pub fn message_routing<S: Into<String>>(msg: S) -> Self {
        Self::MessageRouting(msg.into())
    }
    
    /// 创建协议转换错误
    pub fn protocol_conversion<S: Into<String>>(msg: S) -> Self {
        Self::ProtocolConversion(msg.into())
    }
    
    /// 创建配置错误
    pub fn configuration<S: Into<String>>(msg: S) -> Self {
        Self::Configuration(msg.into())
    }
    
    /// 创建内部错误
    pub fn internal<S: Into<String>>(msg: S) -> Self {
        Self::Internal(msg.into())
    }
    
    /// 检查错误是否可重试
    pub fn is_retryable(&self) -> bool {
        match self {
            GrpcError::Transport(_) => true,
            GrpcError::Status(status) => matches!(
                status.code(),
                Code::Unavailable | Code::DeadlineExceeded | Code::ResourceExhausted
            ),
            GrpcError::MessageRouting(_) => true,
            GrpcError::Internal(_) => true,
            _ => false,
        }
    }
    
    /// 获取错误代码
    pub fn error_code(&self) -> &'static str {
        match self {
            GrpcError::Transport(_) => "TRANSPORT_ERROR",
            GrpcError::Status(_) => "STATUS_ERROR",
            GrpcError::PluginRegistration(_) => "PLUGIN_REGISTRATION_ERROR",
            GrpcError::PluginNotFound { .. } => "PLUGIN_NOT_FOUND",
            GrpcError::AgentRegistration(_) => "AGENT_REGISTRATION_ERROR",
            GrpcError::AgentNotFound { .. } => "AGENT_NOT_FOUND",
            GrpcError::MessageRouting(_) => "MESSAGE_ROUTING_ERROR",
            GrpcError::ProtocolConversion(_) => "PROTOCOL_CONVERSION_ERROR",
            GrpcError::Configuration(_) => "CONFIGURATION_ERROR",
            GrpcError::Serialization(_) => "SERIALIZATION_ERROR",
            GrpcError::A2AProtocol(_) => "A2A_PROTOCOL_ERROR",
            GrpcError::Internal(_) => "INTERNAL_ERROR",
        }
    }
}
