//! 任务管理API处理器
//! 
//! 处理任务的创建、查询、更新和删除操作

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use validator::Validate;
use std::sync::Arc;
use agentx_a2a::{A2ATask, A2AMessage, JsonRpcRequest, MessagePart, FileData, FileWithBytes};

use crate::{
    models::*,
    error::{HttpApiError, HttpApiResult},
    server::AppState,
};

// 这些函数已经在下面定义为pub，不需要重复导出

/// 创建新任务
/// 
/// 根据请求创建一个新的A2A任务
#[utoipa::path(
    post,
    path = "/api/v1/tasks",
    request_body = CreateTaskRequest,
    responses(
        (status = 201, description = "任务创建成功", body = TaskResponse),
        (status = 400, description = "请求参数错误", body = ErrorResponse),
        (status = 500, description = "内部服务器错误", body = ErrorResponse)
    ),
    tag = "tasks"
)]
pub async fn create_task(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateTaskRequest>,
) -> HttpApiResult<(StatusCode, Json<TaskResponse>)> {
    // 验证请求参数
    request.validate()
        .map_err(|e| HttpApiError::ValidationError(format!("参数验证失败: {}", e)))?;
    
    // 创建A2A任务
    let mut task = A2ATask::new(request.kind);
    
    if let Some(context_id) = request.context_id {
        task = task.with_context_id(context_id);
    }
    
    // 添加元数据
    for (key, value) in request.metadata {
        task.metadata.insert(key, value);
    }
    
    // 如果有初始消息，添加到任务中
    if let Some(initial_msg) = request.initial_message {
        let message = convert_create_message_to_a2a(initial_msg)?;
        task = task.add_message(message);
    }
    
    // 通过A2A协议引擎提交任务
    let submit_request = JsonRpcRequest::submit_task(
        task.clone(),
        serde_json::Value::String(uuid::Uuid::new_v4().to_string())
    );
    
    let mut engine = state.engine.lock().await;
    let response = engine.process_request(submit_request).await;
    
    if response.error.is_some() {
        return Err(HttpApiError::InternalError("任务提交失败".to_string()));
    }
    
    // 转换为HTTP响应格式
    let task_response = convert_a2a_task_to_response(&task)?;
    
    Ok((StatusCode::CREATED, Json(task_response)))
}

/// 获取任务详情
/// 
/// 根据任务ID获取任务的详细信息
#[utoipa::path(
    get,
    path = "/api/v1/tasks/{task_id}",
    params(
        ("task_id" = String, Path, description = "任务ID")
    ),
    responses(
        (status = 200, description = "获取任务成功", body = TaskResponse),
        (status = 404, description = "任务不存在", body = ErrorResponse),
        (status = 500, description = "内部服务器错误", body = ErrorResponse)
    ),
    tag = "tasks"
)]
pub async fn get_task(
    State(state): State<Arc<AppState>>,
    Path(task_id): Path<String>,
) -> HttpApiResult<Json<TaskResponse>> {
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
    
    let task: A2ATask = serde_json::from_value(response.result.unwrap())
        .map_err(|e| HttpApiError::InternalError(format!("任务数据解析失败: {}", e)))?;
    
    let task_response = convert_a2a_task_to_response(&task)?;
    
    Ok(Json(task_response))
}

/// 获取任务列表
/// 
/// 分页获取任务列表
#[utoipa::path(
    get,
    path = "/api/v1/tasks",
    params(PaginationQuery),
    responses(
        (status = 200, description = "获取任务列表成功", body = PaginatedResponse<TaskResponse>),
        (status = 400, description = "请求参数错误", body = ErrorResponse),
        (status = 500, description = "内部服务器错误", body = ErrorResponse)
    ),
    tag = "tasks"
)]
pub async fn list_tasks(
    State(_state): State<Arc<AppState>>,
    Query(pagination): Query<PaginationQuery>,
) -> HttpApiResult<Json<PaginatedResponse<TaskResponse>>> {
    // 验证分页参数
    pagination.validate()
        .map_err(|e| HttpApiError::ValidationError(format!("分页参数错误: {}", e)))?;
    
    // TODO: 实现实际的任务列表查询
    // 这里先返回空列表作为示例
    let tasks = Vec::new();
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
        data: tasks,
        pagination: pagination_info,
    };
    
    Ok(Json(response))
}

/// 取消任务
/// 
/// 取消指定的任务
#[utoipa::path(
    post,
    path = "/api/v1/tasks/{task_id}/cancel",
    params(
        ("task_id" = String, Path, description = "任务ID")
    ),
    responses(
        (status = 200, description = "任务取消成功", body = TaskResponse),
        (status = 404, description = "任务不存在", body = ErrorResponse),
        (status = 409, description = "任务状态冲突", body = ErrorResponse),
        (status = 500, description = "内部服务器错误", body = ErrorResponse)
    ),
    tag = "tasks"
)]
pub async fn cancel_task(
    State(state): State<Arc<AppState>>,
    Path(task_id): Path<String>,
) -> HttpApiResult<Json<TaskResponse>> {
    // 通过A2A协议引擎取消任务
    let cancel_request = JsonRpcRequest::new(
        "cancelTask".to_string(),
        Some(serde_json::json!({"taskId": task_id})),
        serde_json::Value::String(uuid::Uuid::new_v4().to_string())
    );
    
    let mut engine = state.engine.lock().await;
    let response = engine.process_request(cancel_request).await;
    
    if let Some(error) = response.error {
        match error.code {
            -32001 => return Err(HttpApiError::NotFound(format!("任务 {} 不存在", task_id))),
            _ => return Err(HttpApiError::InternalError(error.message)),
        }
    }
    
    // 获取更新后的任务信息
    let get_request = JsonRpcRequest::get_task(
        task_id,
        serde_json::Value::String(uuid::Uuid::new_v4().to_string())
    );
    
    let response = engine.process_request(get_request).await;
    let task: A2ATask = serde_json::from_value(response.result.unwrap())
        .map_err(|e| HttpApiError::InternalError(format!("任务数据解析失败: {}", e)))?;
    
    let task_response = convert_a2a_task_to_response(&task)?;
    
    Ok(Json(task_response))
}

// 辅助函数：将CreateMessageRequest转换为A2AMessage
pub fn convert_create_message_to_a2a(request: CreateMessageRequest) -> HttpApiResult<A2AMessage> {
    let mut message = match request.content {
        MessageContent::Text { text } => A2AMessage::new_text(request.role, text),
        MessageContent::File { name, mime_type, data } => {
            let file_data = FileData::WithBytes(FileWithBytes {
                name,
                mime_type,
                bytes: data,
            });
            A2AMessage::new_file(request.role, file_data)
        },
        MessageContent::Data { data } => A2AMessage::new_data(request.role, data),
    };
    
    if let Some(task_id) = request.task_id {
        message = message.with_task_id(task_id);
    }
    
    if let Some(context_id) = request.context_id {
        message = message.with_context_id(context_id);
    }
    
    for (key, value) in request.metadata {
        message = message.with_metadata(key, value);
    }
    
    Ok(message)
}

// 辅助函数：将A2ATask转换为TaskResponse
fn convert_a2a_task_to_response(task: &A2ATask) -> HttpApiResult<TaskResponse> {
    let status = TaskStatusResponse {
        state: task.status.state.clone(),
        timestamp: task.status.timestamp,
        message: task.status.message.clone(),
    };
    
    let artifacts = task.artifacts.iter()
        .map(|artifact| {
            let parts = artifact.parts.iter()
                .map(convert_message_part_to_response)
                .collect::<Result<Vec<_>, _>>()?;
            
            Ok(ArtifactResponse {
                artifact_id: artifact.artifact_id.clone(),
                name: artifact.name.clone(),
                parts,
                metadata: artifact.metadata.clone(),
            })
        })
        .collect::<Result<Vec<_>, HttpApiError>>()?;
    
    let history = task.history.iter()
        .map(convert_a2a_message_to_response)
        .collect::<Result<Vec<_>, _>>()?;
    
    Ok(TaskResponse {
        id: task.id.clone(),
        kind: task.kind.clone(),
        context_id: task.context_id.clone(),
        status,
        artifacts,
        history,
        metadata: task.metadata.clone(),
    })
}

// 辅助函数：将A2AMessage转换为MessageResponse
pub fn convert_a2a_message_to_response(message: &A2AMessage) -> HttpApiResult<MessageResponse> {
    let parts = message.parts.iter()
        .map(convert_message_part_to_response)
        .collect::<Result<Vec<_>, _>>()?;
    
    Ok(MessageResponse {
        message_id: message.message_id.clone(),
        role: message.role.clone(),
        parts,
        task_id: message.task_id.clone(),
        context_id: message.context_id.clone(),
        metadata: message.metadata.clone(),
    })
}

// 辅助函数：将MessagePart转换为MessagePartResponse
fn convert_message_part_to_response(part: &MessagePart) -> HttpApiResult<MessagePartResponse> {
    match part {
        MessagePart::Text(text_part) => Ok(MessagePartResponse::Text {
            text: text_part.text.clone(),
            metadata: text_part.metadata.clone(),
        }),
        MessagePart::File(file_part) => {
            let file_data = match &file_part.file {
                FileData::WithBytes(file_bytes) => FileDataResponse::WithBytes {
                    name: file_bytes.name.clone(),
                    mime_type: file_bytes.mime_type.clone(),
                    bytes: file_bytes.bytes.clone(),
                },
                FileData::WithUri(file_uri) => FileDataResponse::WithUri {
                    name: file_uri.name.clone(),
                    mime_type: file_uri.mime_type.clone(),
                    uri: file_uri.uri.clone(),
                },
            };
            
            Ok(MessagePartResponse::File {
                file: file_data,
                metadata: file_part.metadata.clone(),
            })
        },
        MessagePart::Data(data_part) => Ok(MessagePartResponse::Data {
            data: data_part.data.clone(),
            metadata: data_part.metadata.clone(),
        }),
    }
}
