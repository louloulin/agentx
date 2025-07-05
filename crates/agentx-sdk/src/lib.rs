//! AgentX通用插件SDK
//! 
//! 提供多语言AI框架集成的统一接口和工具
//! 支持LangChain、AutoGen、Mastra、CrewAI、Semantic Kernel等主流框架

pub mod plugin;
pub mod framework;
pub mod adapter;
pub mod builder;
pub mod client;
pub mod server;
pub mod utils;
pub mod lifecycle;
pub mod security;
pub mod config_manager;

// 重新导出核心类型
pub use agentx_a2a::{
    A2AMessage, MessageRole, A2AError, A2AResult,
    StreamManager, StreamType, StreamChunk,
    SecurityManager, SecurityConfig, AuthCredentials, AuthType, TrustLevel,
    MonitoringManager, MonitoringConfig,
};

pub use agentx_grpc::{
    PluginBridge, PluginManager, AgentXGrpcServer, ServerConfig,
};

// SDK核心类型
pub use plugin::{
    Plugin, PluginInfo, PluginStatus, PluginConfig, PluginMetadata,
    PluginCapability, PluginEvent, PluginLifecycle,
};

// 生命周期管理
pub use lifecycle::{
    PluginLifecycleManager, PluginState, HealthStatus, LifecycleConfig,
};

// 安全管理
pub use security::{
    PluginSecurityManager, PermissionPolicy, ResourceLimits, AccessControlList,
    Operation, Resource, SecurityConfig as PluginSecurityConfig, SecurityResult, SecurityAuditEntry,
    create_default_permission_policy,
};

// 配置管理
pub use config_manager::{
    PluginConfigManager, PluginConfigEntry, ConfigValidator, ConfigManagerConfig,
    ConfigFormat, ValidationRule,
};

pub use framework::{
    Framework, FrameworkType, FrameworkAdapter, FrameworkConfig,
    LangChainAdapter, AutoGenAdapter, MastraAdapter,
};

pub use adapter::{
    AgentAdapter, AgentWrapper, AgentProxy, AgentRegistry,
    MessageAdapter, ToolAdapter, WorkflowAdapter,
};

pub use builder::{
    PluginBuilder, FrameworkBuilder, AdapterBuilder,
    ConfigBuilder, ServerBuilder, ClientBuilder,
};

pub use client::{
    PluginClient, FrameworkClient, AgentClient,
    ClientConfig, ConnectionManager, RequestBuilder,
};

pub use server::{
    PluginServer, FrameworkServer, AgentServer,
    ServerManager, ServiceRegistry, EndpointManager,
};

pub use utils::{
    PluginUtils, FrameworkUtils, MessageUtils, ConfigUtils,
    ValidationUtils, ConversionUtils, TestUtils,
};

/// SDK版本信息
pub const SDK_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const SDK_NAME: &str = env!("CARGO_PKG_NAME");

/// 支持的框架列表
pub const SUPPORTED_FRAMEWORKS: &[&str] = &[
    "langchain",
    "autogen", 
    "mastra",
    "crewai",
    "semantic_kernel",
    "langgraph",
    "custom",
];

/// 支持的语言列表
pub const SUPPORTED_LANGUAGES: &[&str] = &[
    "rust",
    "python",
    "nodejs",
    "typescript",
    "csharp",
    "go",
    "java",
];

/// SDK初始化函数
pub async fn init_sdk() -> A2AResult<()> {
    tracing::info!("初始化AgentX SDK v{}", SDK_VERSION);

    // 初始化日志（如果还没有初始化）
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }

    // 尝试初始化tracing，如果已经初始化则忽略错误
    let _ = tracing_subscriber::fmt::try_init();

    tracing::info!("AgentX SDK初始化完成");
    tracing::info!("支持的框架: {:?}", SUPPORTED_FRAMEWORKS);
    tracing::info!("支持的语言: {:?}", SUPPORTED_LANGUAGES);

    Ok(())
}

/// SDK快速启动函数
pub async fn quick_start(framework: &str, config_path: Option<&str>) -> A2AResult<Box<dyn Plugin>> {
    init_sdk().await?;
    
    tracing::info!("快速启动框架: {}", framework);
    
    let config = if let Some(path) = config_path {
        PluginConfig::from_file(path)?
    } else {
        PluginConfig::default_for_framework(framework)?
    };
    
    let plugin = PluginBuilder::new()
        .framework(framework)
        .config(config)
        .build()
        .await?;
    
    tracing::info!("框架 {} 启动成功", framework);
    Ok(plugin)
}

/// 创建插件客户端
pub async fn create_client(server_url: &str) -> A2AResult<PluginClient> {
    let config = ClientConfig::new(server_url);
    PluginClient::new(config).await
}

/// 创建插件服务器
pub async fn create_server(bind_addr: &str) -> A2AResult<PluginServer> {
    let config = ServerConfig {
        host: bind_addr.split(':').next().unwrap_or("127.0.0.1").to_string(),
        port: bind_addr.split(':').nth(1).unwrap_or("50051").parse().unwrap_or(50051),
        max_connections: 1000,
        enable_reflection: true,
        request_timeout_seconds: 30,
    };
    
    PluginServer::new(config).await
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_sdk_init() {
        let result = init_sdk().await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_supported_frameworks() {
        assert!(SUPPORTED_FRAMEWORKS.contains(&"langchain"));
        assert!(SUPPORTED_FRAMEWORKS.contains(&"autogen"));
        assert!(SUPPORTED_FRAMEWORKS.contains(&"mastra"));
    }
    
    #[tokio::test]
    async fn test_supported_languages() {
        assert!(SUPPORTED_LANGUAGES.contains(&"python"));
        assert!(SUPPORTED_LANGUAGES.contains(&"nodejs"));
        assert!(SUPPORTED_LANGUAGES.contains(&"rust"));
    }
}
