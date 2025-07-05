//! HTTP API错误处理
//! 
//! 定义HTTP API的错误类型和错误响应格式

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// HTTP API错误类型
#[derive(Error, Debug)]
pub enum HttpApiError {
    #[error("A2A协议错误: {0}")]
    A2AError(#[from] agentx_a2a::A2AError),
    
    #[error("请求验证失败: {0}")]
    ValidationError(String),
    
    #[error("认证失败: {0}")]
    AuthenticationError(String),
    
    #[error("授权失败: {0}")]
    AuthorizationError(String),
    
    #[error("资源未找到: {0}")]
    NotFound(String),
    
    #[error("请求冲突: {0}")]
    Conflict(String),
    
    #[error("请求过于频繁")]
    RateLimitExceeded,
    
    #[error("内部服务器错误: {0}")]
    InternalError(String),
    
    #[error("服务不可用: {0}")]
    ServiceUnavailable(String),
    
    #[error("JSON序列化错误: {0}")]
    JsonError(#[from] serde_json::Error),
}

/// 标准化的错误响应格式
#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct ErrorResponse {
    /// 错误码
    pub code: String,
    
    /// 错误消息
    pub message: String,
    
    /// 详细信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
    
    /// 请求ID（用于追踪）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    
    /// 时间戳
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl ErrorResponse {
    pub fn new(code: String, message: String) -> Self {
        Self {
            code,
            message,
            details: None,
            request_id: None,
            timestamp: chrono::Utc::now(),
        }
    }
    
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
    
    pub fn with_request_id(mut self, request_id: String) -> Self {
        self.request_id = Some(request_id);
        self
    }
}

impl IntoResponse for HttpApiError {
    fn into_response(self) -> Response {
        let (status, error_response) = match self {
            HttpApiError::ValidationError(msg) => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new("VALIDATION_ERROR".to_string(), msg),
            ),
            HttpApiError::AuthenticationError(msg) => (
                StatusCode::UNAUTHORIZED,
                ErrorResponse::new("AUTHENTICATION_ERROR".to_string(), msg),
            ),
            HttpApiError::AuthorizationError(msg) => (
                StatusCode::FORBIDDEN,
                ErrorResponse::new("AUTHORIZATION_ERROR".to_string(), msg),
            ),
            HttpApiError::NotFound(msg) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new("NOT_FOUND".to_string(), msg),
            ),
            HttpApiError::Conflict(msg) => (
                StatusCode::CONFLICT,
                ErrorResponse::new("CONFLICT".to_string(), msg),
            ),
            HttpApiError::RateLimitExceeded => (
                StatusCode::TOO_MANY_REQUESTS,
                ErrorResponse::new("RATE_LIMIT_EXCEEDED".to_string(), "请求过于频繁，请稍后重试".to_string()),
            ),
            HttpApiError::ServiceUnavailable(msg) => (
                StatusCode::SERVICE_UNAVAILABLE,
                ErrorResponse::new("SERVICE_UNAVAILABLE".to_string(), msg),
            ),
            HttpApiError::A2AError(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse::new("A2A_ERROR".to_string(), err.to_string()),
            ),
            HttpApiError::JsonError(err) => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new("JSON_ERROR".to_string(), err.to_string()),
            ),
            HttpApiError::InternalError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse::new("INTERNAL_ERROR".to_string(), msg),
            ),
        };
        
        (status, Json(error_response)).into_response()
    }
}

/// HTTP API结果类型
pub type HttpApiResult<T> = Result<T, HttpApiError>;
