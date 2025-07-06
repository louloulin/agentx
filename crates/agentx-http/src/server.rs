//! HTTP服务器实现
//! 
//! 基于Axum的HTTP服务器，提供RESTful API接口

use axum::{
    extract::DefaultBodyLimit,
    middleware,
    response::Json,
    routing::{delete, get, post},
    Router,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::timeout::TimeoutLayer;
use tracing::info;

use agentx_a2a::{A2AProtocolEngine, ProtocolEngineConfig};

use crate::{
    config::{HttpServerConfig, AppConfig},
    handlers::{tasks, messages, agents, health, metrics, openapi},
    middleware::*,
    docs::ApiDoc,
    error::HttpApiResult,
};

/// 应用程序状态
pub struct AppState {
    /// A2A协议引擎
    pub engine: Arc<Mutex<A2AProtocolEngine>>,
    
    /// 配置
    pub config: AppConfig,
}

impl AppState {
    /// 创建新的应用程序状态
    pub fn new(config: AppConfig) -> Self {
        let engine_config = ProtocolEngineConfig {
            max_concurrent_tasks: config.a2a.max_concurrent_tasks,
            task_timeout_seconds: config.a2a.task_timeout_seconds,
            enable_message_validation: config.a2a.enable_message_validation,
            enable_task_persistence: config.a2a.enable_task_persistence,
            handler_pool_size: Some(10),
            validate_messages: config.a2a.enable_message_validation,
            max_message_size: 1024 * 1024, // 1MB
        };
        
        let engine = A2AProtocolEngine::new(engine_config);
        
        Self {
            engine: Arc::new(Mutex::new(engine)),
            config,
        }
    }
}

/// HTTP服务器
pub struct HttpServer {
    /// 应用程序状态
    state: Arc<AppState>,
    
    /// 配置
    config: HttpServerConfig,
}

impl HttpServer {
    /// 创建新的HTTP服务器
    pub fn new(config: AppConfig) -> Self {
        let http_config = config.http.clone();
        let state = Arc::new(AppState::new(config));

        Self {
            state,
            config: http_config,
        }
    }

    /// 创建路由（公开方法，用于测试）
    pub fn create_routes(&self) -> Router {
        self.create_routes_internal()
    }
    
    /// 创建路由（内部方法）
    fn create_routes_internal(&self) -> Router {
        let api_routes = Router::new()
            // 任务管理路由
            .route("/tasks", post(tasks::create_task))
            .route("/tasks", get(tasks::list_tasks))
            .route("/tasks/:task_id", get(tasks::get_task))
            .route("/tasks/:task_id/cancel", post(tasks::cancel_task))
            .route("/tasks/:task_id/messages", get(messages::get_task_messages))
            
            // 消息管理路由
            .route("/messages", post(messages::send_message))
            .route("/messages/:message_id", get(messages::get_message))
            
            // Agent管理路由
            .route("/agents", post(agents::register_agent))
            .route("/agents", get(agents::list_agents))
            .route("/agents/:agent_id", get(agents::get_agent))
            .route("/agents/:agent_id", delete(agents::unregister_agent))
            .route("/agents/capabilities", get(agents::get_capabilities))

            // 指标和监控路由
            .route("/metrics", get(metrics::get_metrics))
            .route("/metrics/prometheus", get(metrics::get_prometheus_metrics))
            .route("/metrics/health", get(metrics::get_health_metrics))
            .route("/metrics/performance", get(metrics::get_performance_stats))
            .route("/metrics/reset", post(metrics::reset_metrics))

            // OpenAPI文档路由
            .route("/openapi.json", get(openapi::get_openapi_spec))
            .route("/docs", get(openapi::get_swagger_ui))
            .route("/redoc", get(openapi::get_redoc))
            .route("/openapi/download", get(openapi::download_openapi_spec))
            
            .with_state(self.state.clone());
        
        let mut app = Router::new()
            // API路由
            .nest("/api/v1", api_routes)
            
            // 健康检查路由
            .route("/health", get(health::health_check))
            .route("/ready", get(health::readiness_check))
            .route("/live", get(health::liveness_check))
            
            // API信息路由
            .route("/version", get(Self::version_handler))
            
            .with_state(self.state.clone());
        
        // 添加OpenAPI文档（如果启用）
        if self.config.enable_docs {
            app = app.merge(ApiDoc::swagger_ui());
        }
        
        app
    }
    

    
    /// 启动服务器
    pub async fn start(self) -> Result<(), Box<dyn std::error::Error>> {
        info!("正在启动AgentX HTTP服务器...");
        
        // 验证配置
        self.config.validate()
            .map_err(|e| format!("配置验证失败: {}", e))?;
        
        // 创建路由和中间件
        let app = self.create_routes_internal()
            .layer(middleware::from_fn(request_id_middleware))
            .layer(middleware::from_fn(security_headers_middleware))
            .layer(middleware::from_fn(skip_auth_for_health))
            .layer(middleware::from_fn(request_size_limit_middleware));

        // 应用基础中间件
        let app = if self.config.enable_cors {
            app.layer(cors_middleware())
        } else {
            app
        };

        let app = if self.config.enable_request_logging {
            app.layer(trace_middleware())
        } else {
            app
        };

        let app = if self.config.enable_compression {
            app.layer(compression_middleware())
        } else {
            app
        };

        let app = app
            .layer(TimeoutLayer::new(std::time::Duration::from_secs(
                self.config.request_timeout,
            )))
            .layer(DefaultBodyLimit::max(self.config.max_request_size));
        
        // 获取监听地址
        let addr = self.config.socket_addr();
        
        info!("HTTP服务器启动在 http://{}", addr);
        info!("API文档地址: http://{}/docs", addr);
        info!("健康检查地址: http://{}/health", addr);
        
        // 启动服务器
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;
        
        Ok(())
    }
    
    /// 版本信息处理器
    async fn version_handler() -> HttpApiResult<Json<serde_json::Value>> {
        Ok(Json(serde_json::json!({
            "version": env!("CARGO_PKG_VERSION"),
            "api_version": crate::API_VERSION,
            "build_time": chrono::Utc::now().to_rfc3339(),
        })))
    }
    
    /// OpenAPI文档处理器
    #[allow(dead_code)]
    async fn openapi_handler() -> HttpApiResult<Json<utoipa::openapi::OpenApi>> {
        Ok(Json(ApiDoc::create()))
    }
}

/// 启动HTTP服务器的便捷函数
pub async fn start_server(config: AppConfig) -> Result<(), Box<dyn std::error::Error>> {
    let server = HttpServer::new(config);
    server.start().await
}

/// 使用默认配置启动服务器
pub async fn start_server_with_defaults() -> Result<(), Box<dyn std::error::Error>> {
    let config = AppConfig::default();
    start_server(config).await
}
