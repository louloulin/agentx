//! 健康检查API处理器
//! 
//! 提供系统健康状态检查和监控信息

use axum::{
    extract::State,
    response::Json,
};
use std::sync::Arc;

use crate::{
    models::HealthResponse,
    error::HttpApiResult,
    server::AppState,
};

/// 健康检查
/// 
/// 检查系统的健康状态
#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "系统健康", body = HealthResponse),
        (status = 503, description = "系统不健康", body = HealthResponse)
    ),
    tag = "health"
)]
pub async fn health_check(
    State(state): State<Arc<AppState>>,
) -> HttpApiResult<Json<HealthResponse>> {
    // 检查A2A协议引擎状态
    let engine = state.engine.lock().await;
    let stats = engine.get_stats();
    
    let details = serde_json::json!({
        "a2a_engine": {
            "total_tasks": stats.total_tasks,
            "active_tasks": stats.active_tasks,
            "completed_tasks": stats.completed_tasks,
            "failed_tasks": stats.failed_tasks,
            "messages_processed": stats.messages_processed,
            "messages_routed": stats.messages_routed
        },
        "uptime": "N/A", // TODO: 实现运行时间统计
        "memory_usage": "N/A" // TODO: 实现内存使用统计
    });
    
    let response = HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: chrono::Utc::now(),
        details: Some(details),
    };
    
    Ok(Json(response))
}

/// 就绪检查
/// 
/// 检查系统是否准备好接收请求
#[utoipa::path(
    get,
    path = "/ready",
    responses(
        (status = 200, description = "系统就绪", body = HealthResponse),
        (status = 503, description = "系统未就绪", body = HealthResponse)
    ),
    tag = "health"
)]
pub async fn readiness_check(
    State(_state): State<Arc<AppState>>,
) -> HttpApiResult<Json<HealthResponse>> {
    let response = HealthResponse {
        status: "ready".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: chrono::Utc::now(),
        details: None,
    };
    
    Ok(Json(response))
}

/// 存活检查
/// 
/// 检查系统是否存活
#[utoipa::path(
    get,
    path = "/live",
    responses(
        (status = 200, description = "系统存活", body = HealthResponse)
    ),
    tag = "health"
)]
pub async fn liveness_check() -> HttpApiResult<Json<HealthResponse>> {
    let response = HealthResponse {
        status: "alive".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: chrono::Utc::now(),
        details: None,
    };
    
    Ok(Json(response))
}
