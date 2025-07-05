//! A2A协议使用示例
//! 
//! 本示例展示如何使用AgentX的A2A协议实现进行Agent间通信

use agentx_a2a::*;
use serde_json;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 AgentX A2A协议演示");
    
    // 1. 创建协议引擎
    let config = ProtocolEngineConfig::default();
    let mut engine = A2AProtocolEngine::new(config);
    
    // 2. 注册AI Agent
    let text_agent = AgentInfo {
        id: "text-generator".to_string(),
        name: "文本生成Agent".to_string(),
        endpoint: "http://localhost:8001".to_string(),
        capabilities: vec![
            "text_generation".to_string(),
            "content_writing".to_string(),
            "translation".to_string(),
        ],
        status: AgentStatus::Online,
    };
    
    let analysis_agent = AgentInfo {
        id: "data-analyzer".to_string(),
        name: "数据分析Agent".to_string(),
        endpoint: "http://localhost:8002".to_string(),
        capabilities: vec![
            "data_analysis".to_string(),
            "sentiment_analysis".to_string(),
            "summarization".to_string(),
        ],
        status: AgentStatus::Online,
    };
    
    engine.register_agent(text_agent);
    engine.register_agent(analysis_agent);
    
    println!("✅ 已注册 {} 个Agent", engine.get_stats().total_tasks);
    
    // 3. 创建文本生成任务
    println!("\n📝 创建文本生成任务...");
    
    let user_message = A2AMessage::user_message(
        "请为我生成一篇关于人工智能发展趋势的文章，大约500字".to_string()
    ).with_task_id("task_001".to_string())
     .with_context_id("demo_context".to_string());
    
    let text_task = A2ATask::new("text_generation".to_string())
        .with_context_id("demo_context".to_string())
        .add_message(user_message);
    
    // 4. 提交任务
    let submit_request = JsonRpcRequest::submit_task(
        text_task,
        serde_json::Value::String("req_001".to_string())
    );
    
    let submit_response = engine.process_request(submit_request).await;
    println!("任务提交响应: {}", serde_json::to_string_pretty(&submit_response)?);
    
    // 5. 模拟Agent响应
    println!("\n🤖 模拟Agent生成内容...");
    
    let agent_response = A2AMessage::agent_message(
        "人工智能正在快速发展，从机器学习到深度学习，再到大语言模型，AI技术不断突破...".to_string()
    ).with_task_id("task_001".to_string())
     .with_context_id("demo_context".to_string());
    
    let message_request = JsonRpcRequest::send_message(
        agent_response,
        serde_json::Value::String("req_002".to_string())
    );
    
    let message_response = engine.process_request(message_request).await;
    println!("消息发送响应: {}", serde_json::to_string_pretty(&message_response)?);
    
    // 6. 创建数据分析任务
    println!("\n📊 创建数据分析任务...");
    
    let analysis_data = serde_json::json!({
        "type": "sentiment_analysis",
        "text": "这篇文章写得很好，内容丰富，观点新颖，我很喜欢！",
        "language": "zh-CN",
        "options": {
            "include_confidence": true,
            "detailed_emotions": true
        }
    });
    
    let data_message = A2AMessage::new_data(MessageRole::User, analysis_data)
        .with_task_id("task_002".to_string())
        .with_context_id("demo_context".to_string());
    
    let analysis_task = A2ATask::new("sentiment_analysis".to_string())
        .with_context_id("demo_context".to_string())
        .add_message(data_message);
    
    let analysis_request = JsonRpcRequest::submit_task(
        analysis_task,
        serde_json::Value::String("req_003".to_string())
    );
    
    let analysis_response = engine.process_request(analysis_request).await;
    println!("分析任务响应: {}", serde_json::to_string_pretty(&analysis_response)?);
    
    // 7. 文件处理示例
    println!("\n📁 文件处理示例...");
    
    let file_data = FileData::WithBytes(FileWithBytes {
        name: Some("sample.txt".to_string()),
        mime_type: "text/plain".to_string(),
        bytes: base64_encode("这是一个示例文件内容，用于演示A2A协议的文件处理能力。"),
    });
    
    let file_message = A2AMessage::new_file(MessageRole::User, file_data)
        .with_task_id("task_003".to_string())
        .with_context_id("demo_context".to_string());
    
    let file_request = JsonRpcRequest::send_message(
        file_message,
        serde_json::Value::String("req_004".to_string())
    );
    
    let file_response = engine.process_request(file_request).await;
    println!("文件处理响应: {}", serde_json::to_string_pretty(&file_response)?);
    
    // 8. 查询Agent能力
    println!("\n🔍 查询Agent能力...");
    
    let capabilities_request = JsonRpcRequest::new(
        "getCapabilities".to_string(),
        None,
        serde_json::Value::String("req_005".to_string())
    );
    
    let capabilities_response = engine.process_request(capabilities_request).await;
    println!("能力查询响应: {}", serde_json::to_string_pretty(&capabilities_response)?);
    
    // 9. 错误处理示例
    println!("\n❌ 错误处理示例...");
    
    let invalid_request = JsonRpcRequest::new(
        "invalidMethod".to_string(),
        None,
        serde_json::Value::String("req_006".to_string())
    );
    
    let error_response = engine.process_request(invalid_request).await;
    println!("错误响应: {}", serde_json::to_string_pretty(&error_response)?);
    
    // 10. 显示最终统计信息
    println!("\n📊 最终统计信息:");
    let final_stats = engine.get_stats();
    println!("  总任务数: {}", final_stats.total_tasks);
    println!("  活跃任务数: {}", final_stats.active_tasks);
    println!("  已处理消息数: {}", final_stats.messages_processed);
    println!("  已路由消息数: {}", final_stats.messages_routed);
    
    println!("\n🎉 A2A协议演示完成！");
    
    Ok(())
}

// 辅助函数：使用新的base64 API
fn base64_encode(data: &str) -> String {
    use base64::{Engine as _, engine::general_purpose};
    general_purpose::STANDARD.encode(data.as_bytes())
}
