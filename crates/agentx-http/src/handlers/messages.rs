//! 消息管理API处理器
//! 
//! 处理消息的发送、查询和管理操作

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use validator::Validate;
use std::sync::Arc;
use agentx_a2a::JsonRpcRequest;

use crate::{
    models::*,
    error::{HttpApiError, HttpApiResult},
    server::AppState,
    handlers::tasks,
};

/// 发送消息
/// 
/// 发送一条新消息到指定的任务或上下文
#[utoipa::path(
    post,
    path = "/api/v1/messages",
    request_body = CreateMessageRequest,
    responses(
        (status = 201, description = "消息发送成功", body = MessageResponse),
        (status = 400, description = "请求参数错误", body = ErrorResponse),
        (status = 500, description = "内部服务器错误", body = ErrorResponse)
    ),
    tag = "messages"
)]
pub async fn send_message(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateMessageRequest>,
) -> HttpApiResult<(StatusCode, Json<MessageResponse>)> {
    // 验证请求参数
    request.validate()
        .map_err(|e| HttpApiError::ValidationError(format!("参数验证失败: {}", e)))?;
    
    // 转换为A2A消息
    let message = tasks::convert_create_message_to_a2a(request)?;
    
    // 通过A2A协议引擎发送消息
    let send_request = JsonRpcRequest::send_message(
        message.clone(),
        serde_json::Value::String(uuid::Uuid::new_v4().to_string())
    );
    
    let mut engine = state.engine.lock().await;
    let response = engine.process_request(send_request).await;
    
    if let Some(error) = response.error {
        return Err(HttpApiError::InternalError(format!("消息发送失败: {}", error.message)));
    }
    
    // 转换为HTTP响应格式
    let message_response = tasks::convert_a2a_message_to_response(&message)?;
    
    Ok((StatusCode::CREATED, Json(message_response)))
}

/// 获取消息详情
/// 
/// 根据消息ID获取消息的详细信息
#[utoipa::path(
    get,
    path = "/api/v1/messages/{message_id}",
    params(
        ("message_id" = String, Path, description = "消息ID")
    ),
    responses(
        (status = 200, description = "获取消息成功", body = MessageResponse),
        (status = 404, description = "消息不存在", body = ErrorResponse),
        (status = 500, description = "内部服务器错误", body = ErrorResponse)
    ),
    tag = "messages"
)]
pub async fn get_message(
    State(_state): State<Arc<AppState>>,
    Path(message_id): Path<String>,
) -> HttpApiResult<Json<MessageResponse>> {
    // TODO: 实现消息查询逻辑
    // 这里先返回错误，表示功能待实现
    Err(HttpApiError::NotFound(format!("消息 {} 不存在", message_id)))
}

/// 获取任务的消息历史
/// 
/// 获取指定任务的所有消息历史
#[utoipa::path(
    get,
    path = "/api/v1/tasks/{task_id}/messages",
    params(
        ("task_id" = String, Path, description = "任务ID")
    ),
    responses(
        (status = 200, description = "获取消息历史成功", body = Vec<MessageResponse>),
        (status = 404, description = "任务不存在", body = ErrorResponse),
        (status = 500, description = "内部服务器错误", body = ErrorResponse)
    ),
    tag = "messages"
)]
pub async fn get_task_messages(
    State(state): State<Arc<AppState>>,
    Path(task_id): Path<String>,
) -> HttpApiResult<Json<Vec<MessageResponse>>> {
    // 通过A2A协议引擎查询任务
    let get_request = JsonRpcRequest::get_task(
        task_id.clone(),
        serde_json::Value::String(uuid::Uuid::new_v4().to_string())
    );
    
    let mut engine = state.engine.lock().await;
    let response = engine.process_request(get_request).await;
    
    if let Some(error) = response.error {
        if error.code == -32001 {
            return Err(HttpApiError::NotFound(format!("任务 {} 不存在", task_id)));
        }
        return Err(HttpApiError::InternalError(error.message));
    }
    
    let task: agentx_a2a::A2ATask = serde_json::from_value(response.result.unwrap())
        .map_err(|e| HttpApiError::InternalError(format!("任务数据解析失败: {}", e)))?;
    
    // 转换消息历史
    let messages = task.history.iter()
        .map(tasks::convert_a2a_message_to_response)
        .collect::<Result<Vec<_>, _>>()?;
    
    Ok(Json(messages))
}
