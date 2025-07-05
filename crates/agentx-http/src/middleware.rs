//! HTTP中间件
//! 
//! 提供认证、授权、日志、CORS等中间件功能

use axum::{
    extract::Request,
    http::HeaderMap,
    middleware::Next,
    response::Response,
};
use tower_http::{
    cors::{CorsLayer, Any},
    trace::TraceLayer,
    compression::CompressionLayer,
};
use tracing::{info, warn};

use crate::error::HttpApiError;

/// 创建CORS中间件
pub fn cors_middleware() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
}

/// 创建压缩中间件
pub fn compression_middleware() -> CompressionLayer {
    CompressionLayer::new()
}

/// 创建请求追踪中间件
pub fn trace_middleware() -> TraceLayer<tower_http::classify::SharedClassifier<tower_http::classify::ServerErrorsAsFailures>> {
    TraceLayer::new_for_http()
}

/// API密钥认证中间件
pub async fn api_key_auth(
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, HttpApiError> {
    // 从环境变量获取API密钥
    if let Ok(expected_key) = std::env::var("AGENTX_API_KEY") {
        let auth_header = headers
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "));
        
        match auth_header {
            Some(key) if key == expected_key => {
                info!("API密钥认证成功");
            },
            Some(_) => {
                warn!("API密钥认证失败：密钥无效");
                return Err(HttpApiError::AuthenticationError("无效的API密钥".to_string()));
            },
            None => {
                warn!("API密钥认证失败：缺少Authorization头");
                return Err(HttpApiError::AuthenticationError("缺少API密钥".to_string()));
            },
        }
    }
    
    Ok(next.run(request).await)
}

/// 请求ID中间件
pub async fn request_id_middleware(
    mut request: Request,
    next: Next,
) -> Response {
    // 生成或获取请求ID
    let request_id = if let Some(existing_id) = request
        .headers()
        .get("X-Request-ID")
        .and_then(|h| h.to_str().ok())
    {
        existing_id.to_string()
    } else {
        let id = uuid::Uuid::new_v4().to_string();
        request.headers_mut().insert(
            "X-Request-ID",
            id.parse().unwrap(),
        );
        id
    };

    let mut response = next.run(request).await;

    // 在响应中添加请求ID
    response.headers_mut().insert(
        "X-Request-ID",
        request_id.parse().unwrap(),
    );

    response
}

/// 速率限制中间件
pub async fn rate_limit_middleware(
    request: Request,
    next: Next,
) -> Result<Response, HttpApiError> {
    // TODO: 实现实际的速率限制逻辑
    // 这里先简单通过，后续可以集成Redis或内存存储
    
    Ok(next.run(request).await)
}

/// 请求大小限制中间件
pub async fn request_size_limit_middleware(
    request: Request,
    next: Next,
) -> Result<Response, HttpApiError> {
    // 检查Content-Length头
    if let Some(content_length) = request.headers().get("content-length") {
        if let Ok(length_str) = content_length.to_str() {
            if let Ok(length) = length_str.parse::<usize>() {
                const MAX_SIZE: usize = 10 * 1024 * 1024; // 10MB
                if length > MAX_SIZE {
                    return Err(HttpApiError::ValidationError(
                        format!("请求体过大，最大允许{}字节", MAX_SIZE)
                    ));
                }
            }
        }
    }
    
    Ok(next.run(request).await)
}

/// 安全头中间件
pub async fn security_headers_middleware(
    request: Request,
    next: Next,
) -> Response {
    let mut response = next.run(request).await;
    
    let headers = response.headers_mut();
    
    // 添加安全头
    headers.insert("X-Content-Type-Options", "nosniff".parse().unwrap());
    headers.insert("X-Frame-Options", "DENY".parse().unwrap());
    headers.insert("X-XSS-Protection", "1; mode=block".parse().unwrap());
    headers.insert(
        "Strict-Transport-Security",
        "max-age=31536000; includeSubDomains".parse().unwrap(),
    );
    headers.insert(
        "Content-Security-Policy",
        "default-src 'self'".parse().unwrap(),
    );
    
    response
}

/// 健康检查路径跳过认证中间件
pub async fn skip_auth_for_health(
    request: Request,
    next: Next,
) -> Result<Response, HttpApiError> {
    let path = request.uri().path();
    
    // 健康检查路径跳过认证
    if path.starts_with("/health") || path.starts_with("/ready") || path.starts_with("/live") {
        return Ok(next.run(request).await);
    }
    
    // 文档路径跳过认证
    if path.starts_with("/docs") || path.starts_with("/swagger") {
        return Ok(next.run(request).await);
    }
    
    // 其他路径继续认证流程
    Ok(next.run(request).await)
}
