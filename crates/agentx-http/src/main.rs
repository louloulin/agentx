//! AgentX HTTP服务器主程序
//! 
//! 启动HTTP/REST API服务器

use agentx_http::{config::AppConfig, server::start_server};
use tracing::{info, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    init_logging();
    
    info!("🚀 启动AgentX HTTP服务器");
    
    // 加载配置
    let config = match AppConfig::load() {
        Ok(config) => {
            info!("✅ 配置加载成功");
            config
        },
        Err(e) => {
            error!("❌ 配置加载失败: {}", e);
            info!("使用默认配置");
            AppConfig::default()
        }
    };
    
    // 验证配置
    if let Err(e) = config.validate() {
        error!("❌ 配置验证失败: {}", e);
        std::process::exit(1);
    }
    
    // 打印配置信息
    info!("📋 服务器配置:");
    info!("  监听地址: {}:{}", config.http.host, config.http.port);
    info!("  启用CORS: {}", config.http.enable_cors);
    info!("  启用文档: {}", config.http.enable_docs);
    info!("  最大并发任务: {}", config.a2a.max_concurrent_tasks);
    
    // 启动服务器
    if let Err(e) = start_server(config).await {
        error!("❌ 服务器启动失败: {}", e);
        std::process::exit(1);
    }
    
    Ok(())
}

/// 初始化日志系统
fn init_logging() {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "agentx_http=info,agentx_a2a=info,tower_http=debug".into());
    
    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer())
        .init();
}
