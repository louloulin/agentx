//! 集群错误处理
//! 
//! 定义集群管理的错误类型和处理逻辑

use thiserror::Error;

/// 集群错误类型
#[derive(Error, Debug)]
pub enum ClusterError {
    /// 配置错误
    #[error("配置错误: {0}")]
    ConfigError(String),
    
    /// 网络错误
    #[error("网络错误: {0}")]
    NetworkError(String),
    
    /// 服务发现错误
    #[error("服务发现错误: {0}")]
    ServiceDiscoveryError(String),
    
    /// 负载均衡错误
    #[error("负载均衡错误: {0}")]
    LoadBalancerError(String),
    
    /// 节点管理错误
    #[error("节点管理错误: {0}")]
    NodeManagementError(String),
    
    /// 健康检查错误
    #[error("健康检查错误: {0}")]
    HealthCheckError(String),
    
    /// 状态同步错误
    #[error("状态同步错误: {0}")]
    StateSyncError(String),
    
    /// 不支持的后端
    #[error("不支持的后端: {0}")]
    UnsupportedBackend(String),
    
    /// Agent未找到
    #[error("Agent未找到: {0}")]
    AgentNotFound(String),
    
    /// 节点未找到
    #[error("节点未找到: {0}")]
    NodeNotFound(String),
    
    /// 服务未找到
    #[error("服务未找到: {0}")]
    ServiceNotFound(String),
    
    /// 连接错误
    #[error("连接错误: {0}")]
    ConnectionError(String),
    
    /// 超时错误
    #[error("超时错误: {0}")]
    TimeoutError(String),
    
    /// 认证错误
    #[error("认证错误: {0}")]
    AuthenticationError(String),
    
    /// 权限错误
    #[error("权限错误: {0}")]
    PermissionError(String),
    
    /// 资源不足错误
    #[error("资源不足错误: {0}")]
    ResourceExhaustedError(String),
    
    /// 内部错误
    #[error("内部错误: {0}")]
    InternalError(String),
    
    /// 序列化错误
    #[error("序列化错误: {0}")]
    SerializationError(String),
    
    /// 反序列化错误
    #[error("反序列化错误: {0}")]
    DeserializationError(String),
    
    /// IO错误
    #[error("IO错误: {0}")]
    IoError(String),
    
    /// 数据库错误
    #[error("数据库错误: {0}")]
    DatabaseError(String),
    
    /// 锁错误
    #[error("锁错误: {0}")]
    LockError(String),
    
    /// 任务错误
    #[error("任务错误: {0}")]
    TaskError(String),
}

/// 集群结果类型
pub type ClusterResult<T> = Result<T, ClusterError>;

// 标准错误类型转换
impl From<std::io::Error> for ClusterError {
    fn from(error: std::io::Error) -> Self {
        ClusterError::IoError(error.to_string())
    }
}

impl From<serde_json::Error> for ClusterError {
    fn from(error: serde_json::Error) -> Self {
        ClusterError::SerializationError(error.to_string())
    }
}

impl From<toml::de::Error> for ClusterError {
    fn from(error: toml::de::Error) -> Self {
        ClusterError::DeserializationError(error.to_string())
    }
}

impl From<toml::ser::Error> for ClusterError {
    fn from(error: toml::ser::Error) -> Self {
        ClusterError::SerializationError(error.to_string())
    }
}

impl From<reqwest::Error> for ClusterError {
    fn from(error: reqwest::Error) -> Self {
        if error.is_timeout() {
            ClusterError::TimeoutError(error.to_string())
        } else if error.is_connect() {
            ClusterError::ConnectionError(error.to_string())
        } else {
            ClusterError::NetworkError(error.to_string())
        }
    }
}

impl From<tokio::task::JoinError> for ClusterError {
    fn from(error: tokio::task::JoinError) -> Self {
        ClusterError::TaskError(error.to_string())
    }
}

impl From<uuid::Error> for ClusterError {
    fn from(error: uuid::Error) -> Self {
        ClusterError::InternalError(format!("UUID错误: {}", error))
    }
}

impl From<chrono::ParseError> for ClusterError {
    fn from(error: chrono::ParseError) -> Self {
        ClusterError::DeserializationError(format!("时间解析错误: {}", error))
    }
}

// gRPC错误转换
impl From<tonic::Status> for ClusterError {
    fn from(status: tonic::Status) -> Self {
        match status.code() {
            tonic::Code::NotFound => ClusterError::ServiceNotFound(status.message().to_string()),
            tonic::Code::InvalidArgument => ClusterError::ConfigError(status.message().to_string()),
            tonic::Code::Unauthenticated => ClusterError::AuthenticationError(status.message().to_string()),
            tonic::Code::PermissionDenied => ClusterError::PermissionError(status.message().to_string()),
            tonic::Code::DeadlineExceeded => ClusterError::TimeoutError(status.message().to_string()),
            tonic::Code::ResourceExhausted => ClusterError::ResourceExhaustedError(status.message().to_string()),
            tonic::Code::Unavailable => ClusterError::ConnectionError(status.message().to_string()),
            _ => ClusterError::InternalError(format!("{}: {}", status.code(), status.message())),
        }
    }
}

impl From<ClusterError> for tonic::Status {
    fn from(error: ClusterError) -> Self {
        match error {
            ClusterError::AgentNotFound(msg) | ClusterError::NodeNotFound(msg) | ClusterError::ServiceNotFound(msg) => {
                tonic::Status::not_found(msg)
            }
            ClusterError::ConfigError(msg) => tonic::Status::invalid_argument(msg),
            ClusterError::AuthenticationError(msg) => tonic::Status::unauthenticated(msg),
            ClusterError::PermissionError(msg) => tonic::Status::permission_denied(msg),
            ClusterError::TimeoutError(msg) => tonic::Status::deadline_exceeded(msg),
            ClusterError::ResourceExhaustedError(msg) => tonic::Status::resource_exhausted(msg),
            ClusterError::ConnectionError(msg) | ClusterError::NetworkError(msg) => {
                tonic::Status::unavailable(msg)
            }
            ClusterError::UnsupportedBackend(msg) => tonic::Status::unimplemented(msg),
            _ => tonic::Status::internal(error.to_string()),
        }
    }
}

/// 错误上下文扩展
pub trait ErrorContext<T> {
    /// 添加上下文信息
    fn with_context<F>(self, f: F) -> ClusterResult<T>
    where
        F: FnOnce() -> String;
    
    /// 添加静态上下文信息
    fn context(self, msg: &'static str) -> ClusterResult<T>;
}

impl<T, E> ErrorContext<T> for Result<T, E>
where
    E: Into<ClusterError>,
{
    fn with_context<F>(self, f: F) -> ClusterResult<T>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| {
            let base_error = e.into();
            match base_error {
                ClusterError::InternalError(msg) => {
                    ClusterError::InternalError(format!("{}: {}", f(), msg))
                }
                _ => ClusterError::InternalError(format!("{}: {}", f(), base_error)),
            }
        })
    }
    
    fn context(self, msg: &'static str) -> ClusterResult<T> {
        self.with_context(|| msg.to_string())
    }
}

/// 错误处理工具函数
pub mod utils {
    use super::*;
    use tracing::error;
    
    /// 记录并返回错误
    pub fn log_error<T>(error: ClusterError) -> ClusterResult<T> {
        error!("集群错误: {}", error);
        Err(error)
    }
    
    /// 包装标准错误
    pub fn wrap_error<E: std::error::Error>(error: E, context: &str) -> ClusterError {
        ClusterError::InternalError(format!("{}: {}", context, error))
    }
    
    /// 创建配置错误
    pub fn config_error(msg: &str) -> ClusterError {
        ClusterError::ConfigError(msg.to_string())
    }
    
    /// 创建网络错误
    pub fn network_error(msg: &str) -> ClusterError {
        ClusterError::NetworkError(msg.to_string())
    }
    
    /// 创建服务发现错误
    pub fn service_discovery_error(msg: &str) -> ClusterError {
        ClusterError::ServiceDiscoveryError(msg.to_string())
    }
    
    /// 创建负载均衡错误
    pub fn load_balancer_error(msg: &str) -> ClusterError {
        ClusterError::LoadBalancerError(msg.to_string())
    }
    
    /// 创建节点管理错误
    pub fn node_management_error(msg: &str) -> ClusterError {
        ClusterError::NodeManagementError(msg.to_string())
    }
    
    /// 创建健康检查错误
    pub fn health_check_error(msg: &str) -> ClusterError {
        ClusterError::HealthCheckError(msg.to_string())
    }
    
    /// 创建状态同步错误
    pub fn state_sync_error(msg: &str) -> ClusterError {
        ClusterError::StateSyncError(msg.to_string())
    }
    
    /// 创建Agent未找到错误
    pub fn agent_not_found(agent_id: &str) -> ClusterError {
        ClusterError::AgentNotFound(agent_id.to_string())
    }
    
    /// 创建节点未找到错误
    pub fn node_not_found(node_id: &str) -> ClusterError {
        ClusterError::NodeNotFound(node_id.to_string())
    }
    
    /// 创建服务未找到错误
    pub fn service_not_found(service_id: &str) -> ClusterError {
        ClusterError::ServiceNotFound(service_id.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_display() {
        let error = ClusterError::ConfigError("测试配置错误".to_string());
        assert_eq!(error.to_string(), "配置错误: 测试配置错误");
    }
    
    #[test]
    fn test_error_conversion() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "文件未找到");
        let cluster_error: ClusterError = io_error.into();
        
        match cluster_error {
            ClusterError::IoError(_) => (),
            _ => panic!("错误类型转换失败"),
        }
    }
    
    #[test]
    fn test_error_context() {
        let result: Result<(), std::io::Error> = Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "权限被拒绝"
        ));
        
        let with_context = result.context("读取配置文件时");
        assert!(with_context.is_err());
        
        let error = with_context.unwrap_err();
        assert!(error.to_string().contains("读取配置文件时"));
    }
    
    #[test]
    fn test_tonic_status_conversion() {
        let error = ClusterError::AgentNotFound("test-agent".to_string());
        let status: tonic::Status = error.into();
        assert_eq!(status.code(), tonic::Code::NotFound);
    }
    
    #[test]
    fn test_error_utils() {
        let error = utils::config_error("测试配置错误");
        match error {
            ClusterError::ConfigError(msg) => assert_eq!(msg, "测试配置错误"),
            _ => panic!("错误类型不匹配"),
        }
        
        let error = utils::agent_not_found("test-agent");
        match error {
            ClusterError::AgentNotFound(id) => assert_eq!(id, "test-agent"),
            _ => panic!("错误类型不匹配"),
        }
    }
}
