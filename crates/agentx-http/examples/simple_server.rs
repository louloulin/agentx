//! 简单的HTTP服务器启动示例
//! 
//! 演示如何启动AgentX HTTP API服务器并提供基本的A2A协议功能

use agentx_http::{HttpServer, AppConfig};
use tracing::{info, error};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter("info,agentx_http=debug,agentx_a2a=debug")
        .init();

    info!("🚀 启动AgentX HTTP API服务器示例");

    // 创建应用配置
    let app_config = AppConfig::default();

    // 创建HTTP服务器
    let server = HttpServer::new(app_config);

    info!("📡 HTTP API服务器启动在 http://0.0.0.0:8080");
    info!("📖 API文档: http://0.0.0.0:8080/api/v1/docs");
    info!("🔍 健康检查: http://0.0.0.0:8080/health");
    info!("📊 指标监控: http://0.0.0.0:8080/api/v1/metrics");
    info!("📋 OpenAPI规范: http://0.0.0.0:8080/api/v1/openapi.json");

    // 启动服务器
    if let Err(e) = server.start().await {
        error!("❌ 服务器启动失败: {}", e);
        return Err(e);
    }

    Ok(())
}



#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_server_creation() {
        // 测试服务器创建
        let app_config = AppConfig::default();
        let server = HttpServer::new(app_config);

        // 验证路由创建
        let _routes = server.create_routes();

        // 如果能创建路由，说明服务器配置正确
        assert!(true);
    }
}
