//! LangChain插件演示
//! 
//! 展示如何使用AgentX SDK创建和管理LangChain插件

use agentx_sdk::{
    init_sdk, PluginBuilder, FrameworkBuilder, ConfigBuilder,
    FrameworkType, PluginCapability, PluginUtils, FrameworkUtils,
    A2AMessage, MessageRole, A2AResult,
};
use std::collections::HashMap;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🐍 LangChain插件演示");
    println!("====================");
    
    // 1. 初始化SDK
    println!("\n📦 1. 初始化AgentX SDK");
    init_sdk().await?;
    
    // 2. 检测Python环境
    println!("\n🔍 2. 检测Python环境");
    let env_info = FrameworkUtils::detect_framework_environment("langchain").await?;
    println!("   运行时: {}", env_info.runtime);
    println!("   版本: {}", env_info.version);
    println!("   可用: {}", env_info.available);
    
    if !env_info.available {
        println!("   ⚠️  Python环境不可用，演示将使用模拟模式");
    }
    
    // 3. 创建LangChain配置
    println!("\n⚙️  3. 创建LangChain配置");
    let config = ConfigBuilder::new()
        .framework("langchain")
        .runtime_path("python")
        .working_directory("./examples/langchain")
        .env_var("PYTHONPATH", ".")
        .env_var("OPENAI_API_KEY", "your-api-key-here")
        .custom("model", serde_json::Value::String("gpt-3.5-turbo".to_string()))
        .custom("temperature", serde_json::Value::Number(serde_json::Number::from_f64(0.7).unwrap()))
        .build()?;
    
    println!("   框架: {}", config.framework);
    println!("   绑定地址: {}", config.bind_address);
    println!("   服务器地址: {}", config.server_address);
    
    // 4. 构建LangChain插件
    println!("\n🔧 4. 构建LangChain插件");
    let plugin = PluginBuilder::new()
        .framework("langchain")
        .config(config)
        .capability(PluginCapability::TextProcessing)
        .capability(PluginCapability::ToolCalling)
        .capability(PluginCapability::KnowledgeRetrieval)
        .build()
        .await?;
    
    println!("   插件ID: {}", plugin.get_info().metadata.id);
    println!("   插件名称: {}", plugin.get_info().metadata.name);
    println!("   插件版本: {}", plugin.get_info().metadata.version);
    println!("   支持能力: {:?}", plugin.get_capabilities());
    
    // 5. 创建LangChain框架实例
    println!("\n🏗️  5. 创建LangChain框架实例");
    let framework_config = agentx_sdk::framework::FrameworkConfig {
        framework_type: FrameworkType::LangChain,
        runtime_path: "python".to_string(),
        working_directory: "./examples/langchain".to_string(),
        environment_variables: {
            let mut env = HashMap::new();
            env.insert("PYTHONPATH".to_string(), ".".to_string());
            env.insert("OPENAI_API_KEY".to_string(), "your-api-key-here".to_string());
            env
        },
        startup_args: vec![
            "-c".to_string(),
            "import langchain; print('LangChain loaded successfully')".to_string(),
        ],
        dependencies: vec![
            "langchain".to_string(),
            "langchain-community".to_string(),
            "langchain-openai".to_string(),
        ],
        custom_config: HashMap::new(),
    };
    
    let mut framework = FrameworkBuilder::new()
        .framework_type(FrameworkType::LangChain)
        .config(framework_config)
        .build()
        .await?;
    
    println!("   框架类型: {:?}", framework.get_type());
    println!("   运行状态: {}", framework.is_running());
    
    // 6. 演示消息处理
    println!("\n💬 6. 演示消息处理");
    
    // 创建测试消息
    let test_messages = vec![
        A2AMessage::new_text(MessageRole::User, "Hello, LangChain!".to_string()),
        A2AMessage::new_text(MessageRole::User, "What is artificial intelligence?".to_string()),
        A2AMessage::new_text(MessageRole::User, "Explain machine learning in simple terms.".to_string()),
    ];
    
    for (i, message) in test_messages.iter().enumerate() {
        println!("   📨 处理消息 {}: {}", i + 1, 
            agentx_sdk::MessageUtils::extract_text_content(message));
        
        // 验证消息
        agentx_sdk::MessageUtils::validate_message(message)?;
        
        // 计算消息大小
        let size = agentx_sdk::MessageUtils::calculate_message_size(message);
        println!("     消息大小: {} 字节", size);
        
        // 模拟框架处理（实际环境中会调用真实的LangChain）
        if env_info.available {
            match framework.process_message(message.clone()).await {
                Ok(Some(response)) => {
                    println!("     ✅ 响应: {}", 
                        agentx_sdk::MessageUtils::extract_text_content(&response));
                },
                Ok(None) => {
                    println!("     ℹ️  无响应");
                },
                Err(e) => {
                    println!("     ❌ 处理失败: {:?}", e);
                }
            }
        } else {
            println!("     🔄 模拟响应: Processed by LangChain (simulated)");
        }
    }
    
    // 7. 演示工具调用
    println!("\n🛠️  7. 演示工具调用");
    
    let tool_call_data = serde_json::json!({
        "tool_name": "web_search",
        "arguments": {
            "query": "latest AI news",
            "max_results": 5
        },
        "call_id": "tool_call_001"
    });
    
    let tool_message = A2AMessage::new_data(MessageRole::User, tool_call_data);
    println!("   🔧 工具调用消息: {}", tool_message.message_id);
    
    if env_info.available {
        match framework.process_message(tool_message).await {
            Ok(Some(response)) => {
                println!("   ✅ 工具调用响应: {}", 
                    agentx_sdk::MessageUtils::extract_text_content(&response));
            },
            Ok(None) => {
                println!("   ℹ️  工具调用无响应");
            },
            Err(e) => {
                println!("   ❌ 工具调用失败: {:?}", e);
            }
        }
    } else {
        println!("   🔄 模拟工具调用响应: Web search completed (simulated)");
    }
    
    // 8. 演示插件统计
    println!("\n📊 8. 插件统计信息");
    let stats = plugin.get_stats();
    println!("   处理消息数: {}", stats.messages_processed);
    println!("   发送消息数: {}", stats.messages_sent);
    println!("   接收消息数: {}", stats.messages_received);
    println!("   错误数: {}", stats.errors);
    println!("   平均响应时间: {:.2}ms", stats.avg_response_time_ms);
    
    // 9. 演示配置管理
    println!("\n⚙️  9. 配置管理演示");
    
    // 从环境变量加载配置
    let env_config = agentx_sdk::ConfigUtils::load_from_env();
    println!("   环境配置框架: {}", env_config.framework);
    
    // 合并配置
    let merged_config = agentx_sdk::ConfigUtils::merge_configs(
        plugin.get_info().config.clone(),
        env_config
    );
    println!("   合并后框架: {}", merged_config.framework);
    
    // 10. 清理资源
    println!("\n🧹 10. 清理资源");
    
    if framework.is_running() {
        framework.stop().await?;
        println!("   ✅ LangChain框架已停止");
    }
    
    println!("   ✅ 资源清理完成");
    
    // 11. 总结
    println!("\n📋 11. 演示总结");
    println!("   ✅ SDK初始化成功");
    println!("   ✅ Python环境检测完成");
    println!("   ✅ LangChain配置创建成功");
    println!("   ✅ 插件构建成功");
    println!("   ✅ 框架实例创建成功");
    println!("   ✅ 消息处理演示完成");
    println!("   ✅ 工具调用演示完成");
    println!("   ✅ 统计信息查看完成");
    println!("   ✅ 配置管理演示完成");
    println!("   ✅ 资源清理完成");
    
    println!("\n🎉 LangChain插件演示完成！");
    println!("=====================================");
    
    Ok(())
}

/// 创建示例LangChain脚本
async fn create_example_script() -> A2AResult<()> {
    let script_content = r#"
#!/usr/bin/env python3
"""
LangChain AgentX 插件示例脚本
"""

import sys
import json
from langchain.llms import OpenAI
from langchain.chains import LLMChain
from langchain.prompts import PromptTemplate

def process_message(message_data):
    """处理A2A消息"""
    try:
        # 解析消息
        message = json.loads(message_data)
        content = message.get('content', '')
        
        # 创建LangChain组件
        llm = OpenAI(temperature=0.7)
        prompt = PromptTemplate(
            input_variables=["question"],
            template="You are a helpful AI assistant. Answer the following question: {question}"
        )
        chain = LLMChain(llm=llm, prompt=prompt)
        
        # 处理消息
        response = chain.run(question=content)
        
        # 返回响应
        return {
            "type": "response",
            "content": response,
            "status": "success"
        }
        
    except Exception as e:
        return {
            "type": "error",
            "content": str(e),
            "status": "error"
        }

if __name__ == "__main__":
    if len(sys.argv) > 1:
        message_data = sys.argv[1]
        result = process_message(message_data)
        print(json.dumps(result))
    else:
        print("LangChain AgentX Plugin Ready")
"#;
    
    // 创建示例目录
    tokio::fs::create_dir_all("./examples/langchain").await
        .map_err(|e| agentx_sdk::A2AError::internal(format!("创建目录失败: {}", e)))?;
    
    // 写入脚本文件
    tokio::fs::write("./examples/langchain/plugin.py", script_content).await
        .map_err(|e| agentx_sdk::A2AError::internal(format!("写入脚本失败: {}", e)))?;
    
    println!("   ✅ 示例脚本已创建: ./examples/langchain/plugin.py");
    
    Ok(())
}
