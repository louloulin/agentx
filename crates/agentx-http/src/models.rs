//! HTTP API数据模型
//! 
//! 定义HTTP API的请求和响应数据结构

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;
use agentx_a2a::{A2AMessage, A2ATask, MessageRole, TaskState, AgentStatus};

/// 创建任务请求
#[derive(Serialize, Deserialize, Debug, Validate, ToSchema)]
pub struct CreateTaskRequest {
    /// 任务类型
    #[validate(length(min = 1, max = 100))]
    pub kind: String,
    
    /// 上下文ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_id: Option<String>,
    
    /// 初始消息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_message: Option<CreateMessageRequest>,
    
    /// 任务元数据
    #[serde(default)]
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

/// 创建消息请求
#[derive(Serialize, Deserialize, Debug, Validate, ToSchema)]
pub struct CreateMessageRequest {
    /// 消息角色
    pub role: MessageRole,
    
    /// 消息内容
    pub content: MessageContent,
    
    /// 任务ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
    
    /// 上下文ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_id: Option<String>,
    
    /// 消息元数据
    #[serde(default)]
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

/// 消息内容
#[derive(Serialize, Deserialize, Debug, ToSchema)]
#[serde(tag = "type")]
pub enum MessageContent {
    /// 文本消息
    #[serde(rename = "text")]
    Text {
        /// 文本内容
        text: String,
    },
    
    /// 文件消息
    #[serde(rename = "file")]
    File {
        /// 文件名
        name: Option<String>,
        /// MIME类型
        mime_type: String,
        /// Base64编码的文件内容
        data: String,
    },
    
    /// 结构化数据消息
    #[serde(rename = "data")]
    Data {
        /// JSON数据
        data: serde_json::Value,
    },
}

/// 任务响应
#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct TaskResponse {
    /// 任务ID
    pub id: String,
    
    /// 任务类型
    pub kind: String,
    
    /// 上下文ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_id: Option<String>,
    
    /// 任务状态
    pub status: TaskStatusResponse,
    
    /// 任务工件
    pub artifacts: Vec<ArtifactResponse>,
    
    /// 消息历史
    pub history: Vec<MessageResponse>,
    
    /// 任务元数据
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

/// 任务状态响应
#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct TaskStatusResponse {
    /// 状态
    pub state: TaskState,
    
    /// 时间戳
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<chrono::DateTime<chrono::Utc>>,
    
    /// 状态消息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// 工件响应
#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct ArtifactResponse {
    /// 工件ID
    pub artifact_id: String,
    
    /// 工件名称
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    
    /// 工件内容
    pub parts: Vec<MessagePartResponse>,
    
    /// 工件元数据
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

/// 消息响应
#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct MessageResponse {
    /// 消息ID
    pub message_id: String,
    
    /// 消息角色
    pub role: MessageRole,
    
    /// 消息内容部分
    pub parts: Vec<MessagePartResponse>,
    
    /// 任务ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
    
    /// 上下文ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_id: Option<String>,
    
    /// 消息元数据
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

/// 消息部分响应
#[derive(Serialize, Deserialize, Debug, ToSchema)]
#[serde(tag = "type")]
pub enum MessagePartResponse {
    /// 文本部分
    #[serde(rename = "text")]
    Text {
        /// 文本内容
        text: String,
        /// 元数据
        metadata: std::collections::HashMap<String, serde_json::Value>,
    },
    
    /// 文件部分
    #[serde(rename = "file")]
    File {
        /// 文件数据
        file: FileDataResponse,
        /// 元数据
        metadata: std::collections::HashMap<String, serde_json::Value>,
    },
    
    /// 数据部分
    #[serde(rename = "data")]
    Data {
        /// JSON数据
        data: serde_json::Value,
        /// 元数据
        metadata: std::collections::HashMap<String, serde_json::Value>,
    },
}

/// 文件数据响应
#[derive(Serialize, Deserialize, Debug, ToSchema)]
#[serde(tag = "type")]
pub enum FileDataResponse {
    /// 包含字节数据的文件
    #[serde(rename = "with_bytes")]
    WithBytes {
        /// 文件名
        name: Option<String>,
        /// MIME类型
        mime_type: String,
        /// Base64编码的数据
        bytes: String,
    },
    
    /// 包含URI的文件
    #[serde(rename = "with_uri")]
    WithUri {
        /// 文件名
        name: Option<String>,
        /// MIME类型
        mime_type: String,
        /// 文件URI
        uri: String,
    },
}

/// Agent注册请求
#[derive(Serialize, Deserialize, Debug, Validate, ToSchema)]
pub struct RegisterAgentRequest {
    /// Agent ID
    #[validate(length(min = 1, max = 100))]
    pub id: String,
    
    /// Agent名称
    #[validate(length(min = 1, max = 200))]
    pub name: String,
    
    /// Agent端点
    #[validate(url)]
    pub endpoint: String,
    
    /// Agent能力列表
    pub capabilities: Vec<String>,
    
    /// Agent状态
    #[serde(default)]
    pub status: AgentStatus,
}

/// Agent响应
#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct AgentResponse {
    /// Agent ID
    pub id: String,
    
    /// Agent名称
    pub name: String,
    
    /// Agent端点
    pub endpoint: String,
    
    /// Agent能力列表
    pub capabilities: Vec<String>,
    
    /// Agent状态
    pub status: AgentStatus,
}

/// 分页查询参数
#[derive(Serialize, Deserialize, Debug, Validate, ToSchema)]
#[cfg_attr(feature = "utoipa", derive(utoipa::IntoParams))]
pub struct PaginationQuery {
    /// 页码（从1开始）
    #[validate(range(min = 1))]
    #[serde(default = "default_page")]
    pub page: u32,
    
    /// 每页大小
    #[validate(range(min = 1, max = 100))]
    #[serde(default = "default_page_size")]
    pub page_size: u32,
}

/// 分页响应
#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct PaginatedResponse<T> {
    /// 数据列表
    pub data: Vec<T>,
    
    /// 分页信息
    pub pagination: PaginationInfo,
}

/// 分页信息
#[derive(Serialize, Deserialize, Debug, ToSchema)]
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

/// 健康检查响应
#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct HealthResponse {
    /// 服务状态
    pub status: String,
    
    /// 版本信息
    pub version: String,
    
    /// 时间戳
    pub timestamp: chrono::DateTime<chrono::Utc>,
    
    /// 详细信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

// 默认值函数
fn default_page() -> u32 { 1 }
fn default_page_size() -> u32 { 20 }
