//! HTTP响应类型定义
//! 
//! 定义统一的API响应格式和相关类型

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// 统一的API响应包装器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    /// 请求是否成功
    pub success: bool,
    /// 响应数据（成功时存在）
    pub data: Option<T>,
    /// 错误信息（失败时存在）
    pub error: Option<String>,
    /// 响应时间戳
    pub timestamp: DateTime<Utc>,
    /// 请求唯一标识符
    pub request_id: String,
    /// 分页信息（列表响应时存在）
    pub pagination: Option<PaginationInfo>,
}

/// 分页信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationInfo {
    /// 当前页码
    pub page: u32,
    /// 每页大小
    pub page_size: u32,
    /// 总记录数
    pub total: u64,
    /// 总页数
    pub total_pages: u32,
    /// 是否有下一页
    pub has_next: bool,
    /// 是否有上一页
    pub has_prev: bool,
}

impl<T> ApiResponse<T> {
    /// 创建成功响应
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: Utc::now(),
            request_id: uuid::Uuid::new_v4().to_string(),
            pagination: None,
        }
    }
    
    /// 创建成功响应（带分页信息）
    pub fn success_with_pagination(data: T, pagination: PaginationInfo) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: Utc::now(),
            request_id: uuid::Uuid::new_v4().to_string(),
            pagination: Some(pagination),
        }
    }
    
    /// 创建错误响应
    pub fn error(error: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
            timestamp: Utc::now(),
            request_id: uuid::Uuid::new_v4().to_string(),
            pagination: None,
        }
    }
    
    /// 创建带请求ID的错误响应
    pub fn error_with_id(error: String, request_id: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
            timestamp: Utc::now(),
            request_id,
            pagination: None,
        }
    }
}

impl PaginationInfo {
    /// 创建分页信息
    pub fn new(page: u32, page_size: u32, total: u64) -> Self {
        let total_pages = ((total as f64) / (page_size as f64)).ceil() as u32;
        let has_next = page < total_pages;
        let has_prev = page > 1;
        
        Self {
            page,
            page_size,
            total,
            total_pages,
            has_next,
            has_prev,
        }
    }
    
    /// 计算偏移量
    pub fn offset(&self) -> u64 {
        ((self.page - 1) * self.page_size) as u64
    }
    
    /// 计算限制数量
    pub fn limit(&self) -> u64 {
        self.page_size as u64
    }
}

/// 空响应数据类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmptyResponse;

/// 状态响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusResponse {
    pub status: String,
    pub message: Option<String>,
}

impl StatusResponse {
    pub fn ok() -> Self {
        Self {
            status: "ok".to_string(),
            message: None,
        }
    }
    
    pub fn ok_with_message(message: String) -> Self {
        Self {
            status: "ok".to_string(),
            message: Some(message),
        }
    }
    
    pub fn error(message: String) -> Self {
        Self {
            status: "error".to_string(),
            message: Some(message),
        }
    }
}

/// 计数响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountResponse {
    pub count: u64,
}

impl CountResponse {
    pub fn new(count: u64) -> Self {
        Self { count }
    }
}

/// ID响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdResponse {
    pub id: String,
}

impl IdResponse {
    pub fn new(id: String) -> Self {
        Self { id }
    }
}

/// 版本响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionResponse {
    pub version: String,
    pub build_time: Option<String>,
    pub git_hash: Option<String>,
    pub rust_version: Option<String>,
}

impl VersionResponse {
    pub fn new() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            build_time: option_env!("BUILD_TIME").map(|s| s.to_string()),
            git_hash: option_env!("GIT_HASH").map(|s| s.to_string()),
            rust_version: option_env!("RUST_VERSION").map(|s| s.to_string()),
        }
    }
}

impl Default for VersionResponse {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_api_response_success() {
        let response = ApiResponse::success("test data");
        assert!(response.success);
        assert_eq!(response.data, Some("test data"));
        assert!(response.error.is_none());
        assert!(!response.request_id.is_empty());
    }
    
    #[test]
    fn test_api_response_error() {
        let response: ApiResponse<String> = ApiResponse::error("test error".to_string());
        assert!(!response.success);
        assert!(response.data.is_none());
        assert_eq!(response.error, Some("test error".to_string()));
        assert!(!response.request_id.is_empty());
    }
    
    #[test]
    fn test_pagination_info() {
        let pagination = PaginationInfo::new(2, 10, 25);
        assert_eq!(pagination.page, 2);
        assert_eq!(pagination.page_size, 10);
        assert_eq!(pagination.total, 25);
        assert_eq!(pagination.total_pages, 3);
        assert!(pagination.has_next);
        assert!(pagination.has_prev);
        assert_eq!(pagination.offset(), 10);
        assert_eq!(pagination.limit(), 10);
    }
    
    #[test]
    fn test_status_response() {
        let ok_response = StatusResponse::ok();
        assert_eq!(ok_response.status, "ok");
        assert!(ok_response.message.is_none());
        
        let error_response = StatusResponse::error("test error".to_string());
        assert_eq!(error_response.status, "error");
        assert_eq!(error_response.message, Some("test error".to_string()));
    }
    
    #[test]
    fn test_version_response() {
        let version = VersionResponse::new();
        assert!(!version.version.is_empty());
    }
}
