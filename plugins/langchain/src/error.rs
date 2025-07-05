//! LangChain插件错误处理
//! 
//! 定义LangChain插件的错误类型和处理逻辑

use thiserror::Error;

/// LangChain插件错误类型
#[derive(Error, Debug)]
pub enum LangChainError {
    /// 配置错误
    #[error("配置错误: {0}")]
    ConfigError(String),
    
    /// 环境错误
    #[error("环境错误: {0}")]
    EnvironmentError(String),
    
    /// 网络错误
    #[error("网络错误: {0}")]
    NetworkError(String),
    
    /// API错误
    #[error("API错误: {0}")]
    ApiError(String),
    
    /// 服务错误
    #[error("服务错误: {0}")]
    ServiceError(String),
    
    /// 进程错误
    #[error("进程错误: {0}")]
    ProcessError(String),
    
    /// 序列化错误
    #[error("序列化错误: {0}")]
    SerializationError(String),
    
    /// 无效消息
    #[error("无效消息: {0}")]
    InvalidMessage(String),
    
    /// 无效响应
    #[error("无效响应: {0}")]
    InvalidResponse(String),
    
    /// 不支持的消息类型
    #[error("不支持的消息类型: {0}")]
    UnsupportedMessageType(String),
    
    /// 不支持的操作
    #[error("不支持的操作: {0}")]
    UnsupportedOperation(String),
    
    /// Agent未找到
    #[error("Agent未找到: {0}")]
    AgentNotFound(String),
    
    /// 工具未找到
    #[error("工具未找到: {0}")]
    ToolNotFound(String),
    
    /// 模型未找到
    #[error("模型未找到: {0}")]
    ModelNotFound(String),
    
    /// 会话错误
    #[error("会话错误: {0}")]
    SessionError(String),
    
    /// 流错误
    #[error("流错误: {0}")]
    StreamError(String),
    
    /// 超时错误
    #[error("超时错误: {0}")]
    TimeoutError(String),
    
    /// 认证错误
    #[error("认证错误: {0}")]
    AuthenticationError(String),
    
    /// 权限错误
    #[error("权限错误: {0}")]
    PermissionError(String),
    
    /// 资源限制错误
    #[error("资源限制错误: {0}")]
    ResourceLimitError(String),
    
    /// 内部错误
    #[error("内部错误: {0}")]
    InternalError(String),
    
    /// 服务器错误
    #[error("服务器错误: {0}")]
    ServerError(String),
}

/// LangChain插件结果类型
pub type LangChainResult<T> = Result<T, LangChainError>;

impl From<serde_json::Error> for LangChainError {
    fn from(error: serde_json::Error) -> Self {
        LangChainError::SerializationError(error.to_string())
    }
}

impl From<reqwest::Error> for LangChainError {
    fn from(error: reqwest::Error) -> Self {
        LangChainError::NetworkError(error.to_string())
    }
}

impl From<std::io::Error> for LangChainError {
    fn from(error: std::io::Error) -> Self {
        LangChainError::ProcessError(error.to_string())
    }
}

impl From<toml::de::Error> for LangChainError {
    fn from(error: toml::de::Error) -> Self {
        LangChainError::ConfigError(error.to_string())
    }
}

impl From<toml::ser::Error> for LangChainError {
    fn from(error: toml::ser::Error) -> Self {
        LangChainError::ConfigError(error.to_string())
    }
}

impl From<tonic::Status> for LangChainError {
    fn from(status: tonic::Status) -> Self {
        match status.code() {
            tonic::Code::NotFound => LangChainError::AgentNotFound(status.message().to_string()),
            tonic::Code::InvalidArgument => LangChainError::InvalidMessage(status.message().to_string()),
            tonic::Code::Unauthenticated => LangChainError::AuthenticationError(status.message().to_string()),
            tonic::Code::PermissionDenied => LangChainError::PermissionError(status.message().to_string()),
            tonic::Code::DeadlineExceeded => LangChainError::TimeoutError(status.message().to_string()),
            tonic::Code::ResourceExhausted => LangChainError::ResourceLimitError(status.message().to_string()),
            tonic::Code::Unimplemented => LangChainError::UnsupportedOperation(status.message().to_string()),
            _ => LangChainError::ServerError(format!("{}: {}", status.code(), status.message())),
        }
    }
}

impl From<LangChainError> for tonic::Status {
    fn from(error: LangChainError) -> Self {
        match error {
            LangChainError::AgentNotFound(msg) => tonic::Status::not_found(msg),
            LangChainError::InvalidMessage(msg) => tonic::Status::invalid_argument(msg),
            LangChainError::InvalidResponse(msg) => tonic::Status::invalid_argument(msg),
            LangChainError::AuthenticationError(msg) => tonic::Status::unauthenticated(msg),
            LangChainError::PermissionError(msg) => tonic::Status::permission_denied(msg),
            LangChainError::TimeoutError(msg) => tonic::Status::deadline_exceeded(msg),
            LangChainError::ResourceLimitError(msg) => tonic::Status::resource_exhausted(msg),
            LangChainError::UnsupportedOperation(msg) => tonic::Status::unimplemented(msg),
            LangChainError::UnsupportedMessageType(msg) => tonic::Status::unimplemented(msg),
            _ => tonic::Status::internal(error.to_string()),
        }
    }
}

/// 错误上下文扩展
pub trait ErrorContext<T> {
    /// 添加上下文信息
    fn with_context<F>(self, f: F) -> LangChainResult<T>
    where
        F: FnOnce() -> String;
    
    /// 添加静态上下文信息
    fn context(self, msg: &'static str) -> LangChainResult<T>;
}

impl<T, E> ErrorContext<T> for Result<T, E>
where
    E: Into<LangChainError>,
{
    fn with_context<F>(self, f: F) -> LangChainResult<T>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| {
            let base_error = e.into();
            match base_error {
                LangChainError::InternalError(msg) => {
                    LangChainError::InternalError(format!("{}: {}", f(), msg))
                }
                _ => LangChainError::InternalError(format!("{}: {}", f(), base_error)),
            }
        })
    }
    
    fn context(self, msg: &'static str) -> LangChainResult<T> {
        self.with_context(|| msg.to_string())
    }
}

/// 错误处理工具函数
pub mod utils {
    use super::*;
    use tracing::error;
    
    /// 记录并返回错误
    pub fn log_error<T>(error: LangChainError) -> LangChainResult<T> {
        error!("LangChain插件错误: {}", error);
        Err(error)
    }
    
    /// 包装标准错误
    pub fn wrap_error<E: std::error::Error>(error: E, context: &str) -> LangChainError {
        LangChainError::InternalError(format!("{}: {}", context, error))
    }
    
    /// 创建配置错误
    pub fn config_error(msg: &str) -> LangChainError {
        LangChainError::ConfigError(msg.to_string())
    }
    
    /// 创建网络错误
    pub fn network_error(msg: &str) -> LangChainError {
        LangChainError::NetworkError(msg.to_string())
    }
    
    /// 创建API错误
    pub fn api_error(msg: &str) -> LangChainError {
        LangChainError::ApiError(msg.to_string())
    }
    
    /// 创建服务错误
    pub fn service_error(msg: &str) -> LangChainError {
        LangChainError::ServiceError(msg.to_string())
    }
    
    /// 创建无效消息错误
    pub fn invalid_message(msg: &str) -> LangChainError {
        LangChainError::InvalidMessage(msg.to_string())
    }
    
    /// 创建Agent未找到错误
    pub fn agent_not_found(agent_id: &str) -> LangChainError {
        LangChainError::AgentNotFound(agent_id.to_string())
    }
    
    /// 创建工具未找到错误
    pub fn tool_not_found(tool_name: &str) -> LangChainError {
        LangChainError::ToolNotFound(tool_name.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_display() {
        let error = LangChainError::ConfigError("测试配置错误".to_string());
        assert_eq!(error.to_string(), "配置错误: 测试配置错误");
    }
    
    #[test]
    fn test_error_conversion() {
        let json_error = serde_json::from_str::<serde_json::Value>("invalid json");
        assert!(json_error.is_err());
        
        let langchain_error: LangChainError = json_error.unwrap_err().into();
        match langchain_error {
            LangChainError::SerializationError(_) => (),
            _ => panic!("错误类型转换失败"),
        }
    }
    
    #[test]
    fn test_error_context() {
        let result: Result<(), std::io::Error> = Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "文件未找到"
        ));
        
        let with_context = result.context("读取配置文件时");
        assert!(with_context.is_err());
        
        let error = with_context.unwrap_err();
        assert!(error.to_string().contains("读取配置文件时"));
    }
    
    #[test]
    fn test_tonic_status_conversion() {
        let error = LangChainError::AgentNotFound("test-agent".to_string());
        let status: tonic::Status = error.into();
        assert_eq!(status.code(), tonic::Code::NotFound);
    }
    
    #[test]
    fn test_error_utils() {
        let error = utils::config_error("测试配置错误");
        match error {
            LangChainError::ConfigError(msg) => assert_eq!(msg, "测试配置错误"),
            _ => panic!("错误类型不匹配"),
        }
        
        let error = utils::agent_not_found("test-agent");
        match error {
            LangChainError::AgentNotFound(id) => assert_eq!(id, "test-agent"),
            _ => panic!("错误类型不匹配"),
        }
    }
}
