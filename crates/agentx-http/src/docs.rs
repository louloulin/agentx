//! OpenAPI文档生成
//! 
//! 使用utoipa生成OpenAPI 3.0规范文档

use utoipa::{
    OpenApi,
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
};
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    models::*,
    handlers::{tasks, messages, agents, health},
};

/// OpenAPI文档定义
#[derive(OpenApi)]
#[openapi(
    paths(
        // 任务管理
        tasks::create_task,
        tasks::get_task,
        tasks::list_tasks,
        tasks::cancel_task,
        
        // 消息管理
        messages::send_message,
        messages::get_message,
        messages::get_task_messages,
        
        // Agent管理
        agents::register_agent,
        agents::get_agent,
        agents::list_agents,
        agents::unregister_agent,
        agents::get_capabilities,
        
        // 健康检查
        health::health_check,
        health::readiness_check,
        health::liveness_check,
    ),
    components(
        schemas(
            // 请求模型
            CreateTaskRequest,
            CreateMessageRequest,
            MessageContent,
            RegisterAgentRequest,
            PaginationQuery,
            
            // 响应模型
            TaskResponse,
            TaskStatusResponse,
            ArtifactResponse,
            MessageResponse,
            MessagePartResponse,
            FileDataResponse,
            AgentResponse,
            PaginatedResponse<TaskResponse>,
            PaginatedResponse<AgentResponse>,
            PaginationInfo,
            HealthResponse,
            
            // 错误模型
            crate::error::ErrorResponse,
        )
    ),
    tags(
        (name = "tasks", description = "任务管理API"),
        (name = "messages", description = "消息管理API"),
        (name = "agents", description = "Agent管理API"),
        (name = "health", description = "健康检查API"),
    ),
    info(
        title = "AgentX HTTP API",
        version = "1.0.0",
        description = "AgentX项目的HTTP/REST API服务器，提供A2A协议的HTTP接口",
        contact(
            name = "AgentX Team",
            email = "team@agentx.dev"
        ),
        license(
            name = "MIT",
            url = "https://opensource.org/licenses/MIT"
        )
    ),
    servers(
        (url = "http://localhost:8080", description = "本地开发服务器"),
        (url = "https://api.agentx.dev", description = "生产环境服务器")
    ),
    security(
        ("api_key" = ["read", "write"])
    )
)]
pub struct ApiDoc;

impl ApiDoc {
    /// 创建OpenAPI文档
    pub fn create() -> utoipa::openapi::OpenApi {
        let mut openapi = Self::openapi();
        
        // 添加安全方案
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "api_key",
                SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("Authorization")))
            );
        }
        
        openapi
    }
    
    /// 创建Swagger UI
    pub fn swagger_ui() -> SwaggerUi {
        SwaggerUi::new("/docs")
            .url("/api-docs/openapi.json", Self::create())
    }
}

/// API版本信息
pub fn api_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// API构建信息
pub fn build_info() -> serde_json::Value {
    serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"),
        "build_time": std::env::var("BUILD_TIME").unwrap_or_else(|_| "unknown".to_string()),
        "git_hash": std::env::var("GIT_HASH").unwrap_or_else(|_| "unknown".to_string()),
        "rust_version": std::env::var("RUST_VERSION").unwrap_or_else(|_| "unknown".to_string()),
    })
}
