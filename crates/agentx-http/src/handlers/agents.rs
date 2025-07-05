//! Agent管理API处理器
//! 
//! 处理Agent的注册、查询和管理操作

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use validator::Validate;
use std::sync::Arc;
use agentx_a2a::{AgentInfo, JsonRpcRequest};

use crate::{
    models::*,
    error::{HttpApiError, HttpApiResult},
    server::AppState,
};

/// 注册Agent
/// 
/// 注册一个新的Agent到系统中
#[utoipa::path(
    post,
    path = "/api/v1/agents",
    request_body = RegisterAgentRequest,
    responses(
        (status = 201, description = "Agent注册成功", body = AgentResponse),
        (status = 400, description = "请求参数错误", body = ErrorResponse),
        (status = 409, description = "Agent已存在", body = ErrorResponse),
        (status = 500, description = "内部服务器错误", body = ErrorResponse)
    ),
    tag = "agents"
)]
pub async fn register_agent(
    State(state): State<Arc<AppState>>,
    Json(request): Json<RegisterAgentRequest>,
) -> HttpApiResult<(StatusCode, Json<AgentResponse>)> {
    // 验证请求参数
    request.validate()
        .map_err(|e| HttpApiError::ValidationError(format!("参数验证失败: {}", e)))?;
    
    // 创建AgentInfo
    let agent_info = AgentInfo {
        id: request.id.clone(),
        name: request.name.clone(),
        endpoint: request.endpoint.clone(),
        capabilities: request.capabilities.clone(),
        status: request.status.clone(),
    };
    
    // 注册Agent到协议引擎
    let mut engine = state.engine.lock().await;
    engine.register_agent(agent_info);
    
    // 返回Agent响应
    let agent_response = AgentResponse {
        id: request.id,
        name: request.name,
        endpoint: request.endpoint,
        capabilities: request.capabilities,
        status: request.status,
    };
    
    Ok((StatusCode::CREATED, Json(agent_response)))
}

/// 获取Agent详情
/// 
/// 根据Agent ID获取Agent的详细信息
#[utoipa::path(
    get,
    path = "/api/v1/agents/{agent_id}",
    params(
        ("agent_id" = String, Path, description = "Agent ID")
    ),
    responses(
        (status = 200, description = "获取Agent成功", body = AgentResponse),
        (status = 404, description = "Agent不存在", body = ErrorResponse),
        (status = 500, description = "内部服务器错误", body = ErrorResponse)
    ),
    tag = "agents"
)]
pub async fn get_agent(
    State(_state): State<Arc<AppState>>,
    Path(agent_id): Path<String>,
) -> HttpApiResult<Json<AgentResponse>> {
    // TODO: 实现Agent查询逻辑
    // 这里先返回错误，表示功能待实现
    Err(HttpApiError::NotFound(format!("Agent {} 不存在", agent_id)))
}

/// 获取Agent列表
/// 
/// 分页获取已注册的Agent列表
#[utoipa::path(
    get,
    path = "/api/v1/agents",
    params(PaginationQuery),
    responses(
        (status = 200, description = "获取Agent列表成功", body = PaginatedResponse<AgentResponse>),
        (status = 400, description = "请求参数错误", body = ErrorResponse),
        (status = 500, description = "内部服务器错误", body = ErrorResponse)
    ),
    tag = "agents"
)]
pub async fn list_agents(
    State(_state): State<Arc<AppState>>,
    Query(pagination): Query<PaginationQuery>,
) -> HttpApiResult<Json<PaginatedResponse<AgentResponse>>> {
    // 验证分页参数
    pagination.validate()
        .map_err(|e| HttpApiError::ValidationError(format!("分页参数错误: {}", e)))?;
    
    // TODO: 实现实际的Agent列表查询
    // 这里先返回空列表作为示例
    let agents = Vec::new();
    let total = 0u64;
    
    let pagination_info = PaginationInfo {
        page: pagination.page,
        page_size: pagination.page_size,
        total,
        total_pages: (total as f64 / pagination.page_size as f64).ceil() as u32,
        has_next: pagination.page * pagination.page_size < total as u32,
        has_prev: pagination.page > 1,
    };
    
    let response = PaginatedResponse {
        data: agents,
        pagination: pagination_info,
    };
    
    Ok(Json(response))
}

/// 注销Agent
/// 
/// 从系统中注销指定的Agent
#[utoipa::path(
    delete,
    path = "/api/v1/agents/{agent_id}",
    params(
        ("agent_id" = String, Path, description = "Agent ID")
    ),
    responses(
        (status = 204, description = "Agent注销成功"),
        (status = 404, description = "Agent不存在", body = ErrorResponse),
        (status = 500, description = "内部服务器错误", body = ErrorResponse)
    ),
    tag = "agents"
)]
pub async fn unregister_agent(
    State(state): State<Arc<AppState>>,
    Path(agent_id): Path<String>,
) -> HttpApiResult<StatusCode> {
    // 从协议引擎注销Agent
    let mut engine = state.engine.lock().await;
    engine.unregister_agent(&agent_id);
    
    Ok(StatusCode::NO_CONTENT)
}

/// 获取Agent能力
/// 
/// 获取系统中所有Agent的能力列表
#[utoipa::path(
    get,
    path = "/api/v1/agents/capabilities",
    responses(
        (status = 200, description = "获取能力列表成功", body = Vec<String>),
        (status = 500, description = "内部服务器错误", body = ErrorResponse)
    ),
    tag = "agents"
)]
pub async fn get_capabilities(
    State(state): State<Arc<AppState>>,
) -> HttpApiResult<Json<serde_json::Value>> {
    // 通过A2A协议引擎查询能力
    let capabilities_request = JsonRpcRequest::new(
        "getCapabilities".to_string(),
        None,
        serde_json::Value::String(uuid::Uuid::new_v4().to_string())
    );
    
    let mut engine = state.engine.lock().await;
    let response = engine.process_request(capabilities_request).await;
    
    if let Some(error) = response.error {
        return Err(HttpApiError::InternalError(format!("查询能力失败: {}", error.message)));
    }
    
    Ok(Json(response.result.unwrap()))
}
