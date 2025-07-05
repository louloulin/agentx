//! AgentX HTTP服务器演示
//! 
//! 本示例展示如何启动和使用AgentX的HTTP/REST API服务器

use agentx_http::{
    config::AppConfig,
    server::HttpServer,
    models::*,
};
use reqwest;
use serde_json;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 AgentX HTTP服务器演示");
    
    // 初始化日志
    tracing_subscriber::fmt::init();
    
    // 创建配置
    let mut config = AppConfig::default();
    config.http.port = 8081; // 使用不同的端口避免冲突
    config.http.enable_docs = true;
    
    println!("📋 服务器配置:");
    println!("  监听地址: {}:{}", config.http.host, config.http.port);
    println!("  启用CORS: {}", config.http.enable_cors);
    println!("  启用文档: {}", config.http.enable_docs);
    
    // 在后台启动服务器
    let server_config = config.clone();
    let server_handle = tokio::spawn(async move {
        let server = HttpServer::new(server_config);
        if let Err(e) = server.start().await {
            eprintln!("❌ 服务器启动失败: {}", e);
        }
    });
    
    // 等待服务器启动
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    
    // 创建HTTP客户端
    let client = reqwest::Client::new();
    let base_url = format!("http://{}:{}", config.http.host, config.http.port);
    
    println!("\n🔍 测试HTTP API端点...");
    
    // 1. 测试健康检查
    println!("\n1️⃣ 测试健康检查");
    match client.get(&format!("{}/health", base_url)).send().await {
        Ok(response) => {
            println!("✅ 健康检查状态: {}", response.status());
            if let Ok(text) = response.text().await {
                if let Ok(health) = serde_json::from_str::<HealthResponse>(&text) {
                    println!("   服务状态: {}", health.status);
                    println!("   版本: {}", health.version);
                }
            }
        }
        Err(e) => println!("❌ 健康检查失败: {}", e),
    }
    
    // 2. 测试注册Agent
    println!("\n2️⃣ 测试注册Agent");
    let register_request = RegisterAgentRequest {
        id: "demo_agent".to_string(),
        name: "演示Agent".to_string(),
        endpoint: "http://localhost:8082".to_string(),
        capabilities: vec!["text_generation".to_string(), "translation".to_string()],
        status: agentx_a2a::AgentStatus::Online,
    };
    
    match client
        .post(&format!("{}/api/v1/agents", base_url))
        .json(&register_request)
        .send()
        .await
    {
        Ok(response) => {
            println!("✅ Agent注册状态: {}", response.status());
            if let Ok(text) = response.text().await {
                if let Ok(agent) = serde_json::from_str::<AgentResponse>(&text) {
                    println!("   Agent ID: {}", agent.id);
                    println!("   Agent名称: {}", agent.name);
                    println!("   能力数量: {}", agent.capabilities.len());
                }
            }
        }
        Err(e) => println!("❌ Agent注册失败: {}", e),
    }
    
    // 3. 测试创建任务
    println!("\n3️⃣ 测试创建任务");
    let create_task_request = CreateTaskRequest {
        kind: "text_generation".to_string(),
        context_id: Some("demo_context".to_string()),
        initial_message: Some(CreateMessageRequest {
            role: agentx_a2a::MessageRole::User,
            content: MessageContent::Text {
                text: "请生成一首关于人工智能的诗歌".to_string(),
            },
            task_id: None,
            context_id: None,
            metadata: std::collections::HashMap::new(),
        }),
        metadata: {
            let mut meta = std::collections::HashMap::new();
            meta.insert("priority".to_string(), serde_json::Value::String("high".to_string()));
            meta
        },
    };
    
    match client
        .post(&format!("{}/api/v1/tasks", base_url))
        .json(&create_task_request)
        .send()
        .await
    {
        Ok(response) => {
            println!("✅ 任务创建状态: {}", response.status());
            if let Ok(text) = response.text().await {
                if let Ok(task) = serde_json::from_str::<TaskResponse>(&text) {
                    println!("   任务ID: {}", task.id);
                    println!("   任务类型: {}", task.kind);
                    println!("   任务状态: {:?}", task.status.state);
                    println!("   消息历史: {} 条", task.history.len());
                    
                    // 4. 测试获取任务详情
                    println!("\n4️⃣ 测试获取任务详情");
                    match client
                        .get(&format!("{}/api/v1/tasks/{}", base_url, task.id))
                        .send()
                        .await
                    {
                        Ok(response) => {
                            println!("✅ 任务查询状态: {}", response.status());
                            if let Ok(text) = response.text().await {
                                if let Ok(retrieved_task) = serde_json::from_str::<TaskResponse>(&text) {
                                    println!("   查询到的任务ID: {}", retrieved_task.id);
                                    println!("   任务状态: {:?}", retrieved_task.status.state);
                                }
                            }
                        }
                        Err(e) => println!("❌ 任务查询失败: {}", e),
                    }
                }
            }
        }
        Err(e) => println!("❌ 任务创建失败: {}", e),
    }
    
    // 5. 测试发送消息
    println!("\n5️⃣ 测试发送消息");
    let message_request = CreateMessageRequest {
        role: agentx_a2a::MessageRole::Agent,
        content: MessageContent::Text {
            text: "我已经为您生成了一首关于AI的诗歌...".to_string(),
        },
        task_id: Some("demo_task_123".to_string()),
        context_id: Some("demo_context".to_string()),
        metadata: std::collections::HashMap::new(),
    };
    
    match client
        .post(&format!("{}/api/v1/messages", base_url))
        .json(&message_request)
        .send()
        .await
    {
        Ok(response) => {
            println!("✅ 消息发送状态: {}", response.status());
            if let Ok(text) = response.text().await {
                if let Ok(message) = serde_json::from_str::<MessageResponse>(&text) {
                    println!("   消息ID: {}", message.message_id);
                    println!("   消息角色: {:?}", message.role);
                    println!("   消息部分数: {}", message.parts.len());
                }
            }
        }
        Err(e) => println!("❌ 消息发送失败: {}", e),
    }
    
    // 6. 测试获取Agent能力
    println!("\n6️⃣ 测试获取Agent能力");
    match client
        .get(&format!("{}/api/v1/agents/capabilities", base_url))
        .send()
        .await
    {
        Ok(response) => {
            println!("✅ 能力查询状态: {}", response.status());
            if let Ok(text) = response.text().await {
                if let Ok(capabilities) = serde_json::from_str::<serde_json::Value>(&text) {
                    println!("   能力信息: {}", serde_json::to_string_pretty(&capabilities).unwrap_or_default());
                }
            }
        }
        Err(e) => println!("❌ 能力查询失败: {}", e),
    }
    
    // 7. 测试文档端点（如果启用）
    if config.http.enable_docs {
        println!("\n7️⃣ 测试API文档");
        match client
            .get(&format!("{}/docs", base_url))
            .send()
            .await
        {
            Ok(response) => {
                println!("✅ API文档状态: {}", response.status());
                println!("   文档地址: {}/docs", base_url);
            }
            Err(e) => println!("❌ API文档访问失败: {}", e),
        }
    }
    
    println!("\n🎉 HTTP API演示完成！");
    println!("📖 API文档地址: {}/docs", base_url);
    println!("🔍 健康检查地址: {}/health", base_url);
    
    // 保持服务器运行一段时间
    println!("\n⏰ 服务器将继续运行30秒，您可以手动测试API...");
    tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
    
    // 停止服务器
    server_handle.abort();
    println!("🛑 服务器已停止");
    
    Ok(())
}
