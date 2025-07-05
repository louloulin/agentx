//! 路由器错误类型定义
//! 
//! 定义路由器相关的错误类型和处理逻辑

use thiserror::Error;

/// 路由器错误类型
#[derive(Error, Debug)]
pub enum RouterError {
    /// 没有可用的Agent
    #[error("没有可用的Agent: {0}")]
    NoAvailableAgents(String),

    /// 没有可用的端点
    #[error("没有可用的端点")]
    NoAvailableEndpoints,

    /// 路由失败
    #[error("路由失败 - Agent: {agent_id}, 尝试次数: {attempts}, 最后错误: {last_error:?}")]
    RoutingFailed {
        agent_id: String,
        attempts: u32,
        last_error: Option<String>,
    },

    /// 无效的路由策略
    #[error("无效的路由策略: {0}")]
    InvalidStrategy(String),

    /// 连接超时
    #[error("连接超时: {0}")]
    ConnectionTimeout(String),

    /// 网络错误
    #[error("网络错误: {0}")]
    NetworkError(String),

    /// 协议错误
    #[error("协议错误: {0}")]
    ProtocolError(String),

    /// 认证失败
    #[error("认证失败: {0}")]
    AuthenticationFailed(String),

    /// 授权失败
    #[error("授权失败: {0}")]
    AuthorizationFailed(String),

    /// 服务不可用
    #[error("服务不可用: {0}")]
    ServiceUnavailable(String),

    /// 内部错误
    #[error("内部错误: {0}")]
    InternalError(String),

    /// A2A协议错误
    #[error("A2A协议错误: {0}")]
    A2AError(#[from] agentx_a2a::A2AError),

    /// 序列化错误
    #[error("序列化错误: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// IO错误
    #[error("IO错误: {0}")]
    IoError(#[from] std::io::Error),
}

impl RouterError {
    /// 检查是否为连接性错误
    pub fn is_connectivity_error(&self) -> bool {
        matches!(
            self,
            RouterError::ConnectionTimeout(_)
                | RouterError::NetworkError(_)
                | RouterError::ServiceUnavailable(_)
        )
    }

    /// 检查是否为临时错误（可重试）
    pub fn is_temporary(&self) -> bool {
        matches!(
            self,
            RouterError::ConnectionTimeout(_)
                | RouterError::NetworkError(_)
                | RouterError::ServiceUnavailable(_)
                | RouterError::InternalError(_)
        )
    }

    /// 检查是否为认证/授权错误
    pub fn is_auth_error(&self) -> bool {
        matches!(
            self,
            RouterError::AuthenticationFailed(_) | RouterError::AuthorizationFailed(_)
        )
    }

    /// 获取错误代码
    pub fn error_code(&self) -> &'static str {
        match self {
            RouterError::NoAvailableAgents(_) => "NO_AGENTS",
            RouterError::NoAvailableEndpoints => "NO_ENDPOINTS",
            RouterError::RoutingFailed { .. } => "ROUTING_FAILED",
            RouterError::InvalidStrategy(_) => "INVALID_STRATEGY",
            RouterError::ConnectionTimeout(_) => "CONNECTION_TIMEOUT",
            RouterError::NetworkError(_) => "NETWORK_ERROR",
            RouterError::ProtocolError(_) => "PROTOCOL_ERROR",
            RouterError::AuthenticationFailed(_) => "AUTH_FAILED",
            RouterError::AuthorizationFailed(_) => "AUTHZ_FAILED",
            RouterError::ServiceUnavailable(_) => "SERVICE_UNAVAILABLE",
            RouterError::InternalError(_) => "INTERNAL_ERROR",
            RouterError::A2AError(_) => "A2A_ERROR",
            RouterError::SerializationError(_) => "SERIALIZATION_ERROR",
            RouterError::IoError(_) => "IO_ERROR",
        }
    }

    /// 获取HTTP状态码
    pub fn http_status_code(&self) -> u16 {
        match self {
            RouterError::NoAvailableAgents(_) | RouterError::NoAvailableEndpoints => 503,
            RouterError::RoutingFailed { .. } => 502,
            RouterError::InvalidStrategy(_) => 400,
            RouterError::ConnectionTimeout(_) => 504,
            RouterError::NetworkError(_) => 502,
            RouterError::ProtocolError(_) => 400,
            RouterError::AuthenticationFailed(_) => 401,
            RouterError::AuthorizationFailed(_) => 403,
            RouterError::ServiceUnavailable(_) => 503,
            RouterError::InternalError(_) => 500,
            RouterError::A2AError(_) => 500,
            RouterError::SerializationError(_) => 400,
            RouterError::IoError(_) => 500,
        }
    }

    /// 创建连接超时错误
    pub fn connection_timeout(endpoint: &str) -> Self {
        RouterError::ConnectionTimeout(format!("连接到 {} 超时", endpoint))
    }

    /// 创建网络错误
    pub fn network_error(message: &str) -> Self {
        RouterError::NetworkError(message.to_string())
    }

    /// 创建协议错误
    pub fn protocol_error(message: &str) -> Self {
        RouterError::ProtocolError(message.to_string())
    }

    /// 创建服务不可用错误
    pub fn service_unavailable(service: &str) -> Self {
        RouterError::ServiceUnavailable(format!("服务 {} 不可用", service))
    }

    /// 创建内部错误
    pub fn internal_error(message: &str) -> Self {
        RouterError::InternalError(message.to_string())
    }
}

/// 路由器结果类型
pub type RouterResult<T> = Result<T, RouterError>;

/// 错误统计信息
#[derive(Debug, Clone, Default)]
pub struct ErrorStats {
    /// 总错误数
    pub total_errors: u64,
    /// 连接错误数
    pub connection_errors: u64,
    /// 网络错误数
    pub network_errors: u64,
    /// 协议错误数
    pub protocol_errors: u64,
    /// 认证错误数
    pub auth_errors: u64,
    /// 内部错误数
    pub internal_errors: u64,
    /// 最后错误时间
    pub last_error_time: Option<chrono::DateTime<chrono::Utc>>,
}

impl ErrorStats {
    /// 记录错误
    pub fn record_error(&mut self, error: &RouterError) {
        self.total_errors += 1;
        self.last_error_time = Some(chrono::Utc::now());

        match error {
            RouterError::ConnectionTimeout(_) | RouterError::NetworkError(_) => {
                self.connection_errors += 1;
                self.network_errors += 1;
            }
            RouterError::ProtocolError(_) => {
                self.protocol_errors += 1;
            }
            RouterError::AuthenticationFailed(_) | RouterError::AuthorizationFailed(_) => {
                self.auth_errors += 1;
            }
            RouterError::InternalError(_) | RouterError::A2AError(_) => {
                self.internal_errors += 1;
            }
            _ => {}
        }
    }

    /// 获取错误率
    pub fn error_rate(&self, total_requests: u64) -> f64 {
        if total_requests == 0 {
            0.0
        } else {
            self.total_errors as f64 / total_requests as f64
        }
    }

    /// 重置统计
    pub fn reset(&mut self) {
        *self = ErrorStats::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_classification() {
        let timeout_error = RouterError::connection_timeout("http://example.com");
        assert!(timeout_error.is_connectivity_error());
        assert!(timeout_error.is_temporary());
        assert!(!timeout_error.is_auth_error());

        let auth_error = RouterError::AuthenticationFailed("Invalid token".to_string());
        assert!(!auth_error.is_connectivity_error());
        assert!(!auth_error.is_temporary());
        assert!(auth_error.is_auth_error());
    }

    #[test]
    fn test_error_stats() {
        let mut stats = ErrorStats::default();
        
        let error1 = RouterError::connection_timeout("http://example.com");
        let error2 = RouterError::AuthenticationFailed("Invalid token".to_string());
        
        stats.record_error(&error1);
        stats.record_error(&error2);
        
        assert_eq!(stats.total_errors, 2);
        assert_eq!(stats.connection_errors, 1);
        assert_eq!(stats.auth_errors, 1);
        assert!(stats.last_error_time.is_some());
    }
}
