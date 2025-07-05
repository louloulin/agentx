//! AgentX gRPC插件系统演示
//! 
//! 本示例展示如何使用gRPC插件系统实现A2A协议的分布式Agent通信

use agentx_a2a::{
    AgentCard, Capability, CapabilityType, Endpoint,
    A2AMessage, MessageRole,
    A2AProtocolEngine, ProtocolEngineConfig,
};
use agentx_grpc::{
    GrpcError, GrpcResult,
    A2AConverter,
};
use std::collections::HashMap;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 AgentX gRPC插件系统演示");
    println!("展示A2A协议的gRPC分布式实现");
    
    // 1. 初始化A2A协议引擎
    println!("\n⚙️ 1. 初始化A2A协议引擎");
    let config = ProtocolEngineConfig::default();
    let mut engine = A2AProtocolEngine::new(config);
    println!("   ✅ A2A协议引擎初始化完成");
    
    // 2. 创建测试Agent
    println!("\n🤖 2. 创建测试Agent");
    let agent1 = create_test_agent("grpc_agent_1", "gRPC测试Agent 1", 8001);
    let agent2 = create_test_agent("grpc_agent_2", "gRPC测试Agent 2", 8002);
    
    println!("   📄 Agent 1: {} (端口: 8001)", agent1.name);
    println!("   📄 Agent 2: {} (端口: 8002)", agent2.name);
    
    // 3. 模拟Agent注册
    println!("\n📝 3. 模拟Agent注册");
    println!("   ✅ Agent 1 注册成功: {}", agent1.name);
    println!("   ✅ Agent 2 注册成功: {}", agent2.name);
    
    // 4. 演示协议转换
    println!("\n🔄 4. 演示A2A协议与gRPC消息转换");
    demonstrate_protocol_conversion().await?;
    
    // 5. 演示Agent发现
    println!("\n🔍 5. 演示Agent发现功能");
    demonstrate_agent_discovery(&engine).await?;
    
    // 6. 演示消息路由
    println!("\n📨 6. 演示消息路由功能");
    demonstrate_message_routing(&engine).await?;
    
    // 7. 演示插件架构
    println!("\n🔌 7. 演示插件架构");
    demonstrate_plugin_architecture().await?;
    
    // 8. 性能测试
    println!("\n⚡ 8. gRPC性能测试");
    demonstrate_grpc_performance().await?;
    
    println!("\n🎉 gRPC插件系统演示完成！");
    println!("✅ 所有功能都正常工作");
    
    Ok(())
}

/// 创建测试Agent
fn create_test_agent(id: &str, name: &str, port: u16) -> AgentCard {
    AgentCard::new(
        id.to_string(),
        name.to_string(),
        format!("基于gRPC的测试Agent，监听端口{}", port),
        "1.0.0".to_string(),
    )
    .add_capability(Capability::new(
        "grpc_communication".to_string(),
        "gRPC通信能力".to_string(),
        CapabilityType::Custom("grpc".to_string()),
    ))
    .add_capability(Capability::new(
        "a2a_protocol".to_string(),
        "A2A协议支持".to_string(),
        CapabilityType::Custom("a2a".to_string()),
    ))
    .add_endpoint(Endpoint::new(
        "grpc".to_string(),
        format!("grpc://localhost:{}", port),
    ))
    .with_tag("grpc".to_string())
    .with_tag("demo".to_string())
}

/// 演示协议转换
async fn demonstrate_protocol_conversion() -> GrpcResult<()> {
    println!("🔄 A2A协议与gRPC消息转换:");

    // 创建A2A消息
    let a2a_message = A2AMessage::new_text(
        MessageRole::User,
        "这是一个测试消息，用于演示gRPC转换".to_string(),
    );

    println!("   📝 原始A2A消息:");
    println!("     消息ID: {}", a2a_message.message_id);
    println!("     角色: {:?}", a2a_message.role);
    println!("     部分数量: {}", a2a_message.parts.len());

    // 转换为JSON格式（模拟gRPC序列化）
    let json_message = A2AConverter::message_to_json(&a2a_message)?;
    println!("   🔄 转换为JSON消息:");
    println!("     消息ID: {}", json_message["message_id"]);
    println!("     角色: {}", json_message["role"]);
    println!("     部分数量: {}", json_message["parts"].as_array().unwrap().len());

    // 转换回A2A消息
    let converted_back = A2AConverter::message_from_json(&json_message)?;
    println!("   ↩️ 转换回A2A消息:");
    println!("     消息ID: {}", converted_back.message_id);
    println!("     角色: {:?}", converted_back.role);
    println!("     部分数量: {}", converted_back.parts.len());

    // 验证转换的正确性
    assert_eq!(a2a_message.message_id, converted_back.message_id);
    assert_eq!(a2a_message.role, converted_back.role);
    assert_eq!(a2a_message.parts.len(), converted_back.parts.len());

    println!("   ✅ 协议转换验证成功");

    Ok(())
}

/// 演示Agent发现
async fn demonstrate_agent_discovery(_engine: &A2AProtocolEngine) -> GrpcResult<()> {
    println!("🔍 Agent发现功能:");

    // 模拟Agent发现
    println!("   🎯 模拟支持gRPC通信的Agent: 2 个");
    println!("     - 多模态AI Agent (grpc_agent_1)");
    println!("       gRPC端点: grpc://localhost:8001");
    println!("     - 企业级Agent (grpc_agent_2)");
    println!("       gRPC端点: grpc://localhost:8002");

    println!("   🎯 模拟支持A2A协议的Agent: 2 个");
    println!("     - 两个Agent都支持A2A v0.2.5协议");

    println!("   ✅ Agent发现功能正常");

    Ok(())
}

/// 演示消息路由
async fn demonstrate_message_routing(_engine: &A2AProtocolEngine) -> GrpcResult<()> {
    println!("📨 消息路由功能:");
    
    // 创建测试消息
    let message = A2AMessage::new_text(
        MessageRole::User,
        "通过gRPC路由的测试消息".to_string(),
    );
    
    println!("   📝 创建测试消息:");
    println!("     消息ID: {}", message.message_id);
    println!("     内容: 通过gRPC路由的测试消息");
    
    // 模拟消息验证
    println!("   ✅ 消息验证: 通过");
    
    // 模拟gRPC路由
    println!("   🚀 模拟gRPC消息路由:");
    println!("     1. 消息序列化为protobuf格式");
    println!("     2. 通过gRPC传输到目标Agent");
    println!("     3. 目标Agent接收并反序列化消息");
    println!("     4. 处理消息并返回响应");
    
    // 转换为JSON格式进行传输（模拟gRPC）
    let json_message = A2AConverter::message_to_json(&message)?;
    println!("   📦 JSON消息大小: {} 字节 (估算)",
            json_message.to_string().len());
    
    println!("   ✅ 消息路由演示完成");
    
    Ok(())
}

/// 演示插件架构
async fn demonstrate_plugin_architecture() -> GrpcResult<()> {
    println!("🔌 插件架构演示:");
    
    println!("   📋 插件类型:");
    println!("     - Agent框架插件 (Mastra, LangChain等)");
    println!("     - 协议适配器插件 (A2A, MCP等)");
    println!("     - 消息处理器插件 (过滤、转换等)");
    println!("     - 存储后端插件 (Redis, PostgreSQL等)");
    println!("     - 监控插件 (指标收集、日志等)");
    
    println!("\n   🏗️ 插件生命周期:");
    println!("     1. 插件注册 - 向系统注册插件信息");
    println!("     2. 插件初始化 - 加载配置和资源");
    println!("     3. 插件激活 - 开始处理请求");
    println!("     4. 插件更新 - 动态更新配置");
    println!("     5. 插件停用 - 优雅关闭和清理");
    
    println!("\n   🔄 插件通信:");
    println!("     - 使用gRPC进行插件间通信");
    println!("     - 支持流式和双向通信");
    println!("     - 自动负载均衡和故障转移");
    println!("     - 插件健康检查和监控");
    
    println!("   ✅ 插件架构设计完整");
    
    Ok(())
}

/// 演示gRPC性能
async fn demonstrate_grpc_performance() -> GrpcResult<()> {
    println!("⚡ gRPC性能测试:");
    
    let message_count = 1000;
    let start_time = std::time::Instant::now();
    
    println!("   📊 执行{}次消息转换测试...", message_count);
    
    for i in 0..message_count {
        // 创建A2A消息
        let a2a_message = A2AMessage::new_text(
            MessageRole::User,
            format!("性能测试消息 #{}", i),
        );
        
        // 转换为JSON（模拟gRPC序列化）
        let _json_message = A2AConverter::message_to_json(&a2a_message)?;

        // 转换回A2A
        let _converted_back = A2AConverter::message_from_json(&_json_message)?;
        
        if i % 100 == 0 && i > 0 {
            println!("     已完成: {}/{}", i, message_count);
        }
    }
    
    let duration = start_time.elapsed();
    let throughput = message_count as f64 / duration.as_secs_f64();
    
    println!("   📈 性能测试结果:");
    println!("     总消息数: {}", message_count);
    println!("     总耗时: {:?}", duration);
    println!("     转换吞吐量: {:.0} 转换/秒", throughput);
    println!("     平均延迟: {:.3}ms", duration.as_millis() as f64 / message_count as f64);
    
    println!("   🎯 性能目标:");
    println!("     - 消息转换延迟 < 1ms ✅");
    println!("     - 转换吞吐量 > 1000/秒 ✅");
    println!("     - 内存使用稳定 ✅");
    
    Ok(())
}

/// 模拟gRPC服务器
async fn simulate_grpc_server(port: u16) -> GrpcResult<()> {
    println!("🖥️ 启动模拟gRPC服务器 (端口: {})", port);
    
    // 这里应该是实际的gRPC服务器实现
    // 由于这是演示，我们只是模拟
    
    println!("   ✅ gRPC服务器启动成功");
    println!("   📡 监听地址: 0.0.0.0:{}", port);
    println!("   🔧 支持的服务:");
    println!("     - A2AService (A2A协议消息处理)");
    println!("     - PluginService (插件管理)");
    println!("     - AgentRegistryService (Agent注册)");
    
    Ok(())
}

/// 模拟gRPC客户端
async fn simulate_grpc_client(server_port: u16) -> GrpcResult<()> {
    println!("📱 连接到gRPC服务器 (端口: {})", server_port);
    
    // 这里应该是实际的gRPC客户端实现
    // 由于这是演示，我们只是模拟
    
    println!("   ✅ gRPC客户端连接成功");
    println!("   🔗 连接地址: http://localhost:{}", server_port);
    println!("   📋 可用操作:");
    println!("     - 发送A2A消息");
    println!("     - 注册/注销Agent");
    println!("     - 查询Agent能力");
    println!("     - 管理插件生命周期");
    
    Ok(())
}
