//! AutoGen插件演示
//! 
//! 展示如何使用AgentX SDK创建和管理AutoGen多Agent对话插件

use agentx_sdk::{
    init_sdk, PluginBuilder, FrameworkBuilder, ConfigBuilder,
    FrameworkType, PluginCapability, FrameworkUtils,
    A2AMessage, MessageRole, A2AResult,
};
use std::collections::HashMap;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 AutoGen多Agent插件演示");
    println!("==========================");
    
    // 1. 初始化SDK
    println!("\n📦 1. 初始化AgentX SDK");
    init_sdk().await?;
    
    // 2. 检测Python环境
    println!("\n🔍 2. 检测Python环境");
    let env_info = FrameworkUtils::detect_framework_environment("autogen").await?;
    println!("   运行时: {}", env_info.runtime);
    println!("   版本: {}", env_info.version);
    println!("   可用: {}", env_info.available);
    
    if !env_info.available {
        println!("   ⚠️  Python环境不可用，演示将使用模拟模式");
    }
    
    // 3. 创建AutoGen配置
    println!("\n⚙️  3. 创建AutoGen配置");
    let config = ConfigBuilder::new()
        .framework("autogen")
        .runtime_path("python")
        .working_directory("./examples/autogen")
        .env_var("PYTHONPATH", ".")
        .env_var("OPENAI_API_KEY", "your-api-key-here")
        .custom("max_agents", serde_json::Value::Number(serde_json::Number::from(5)))
        .custom("conversation_rounds", serde_json::Value::Number(serde_json::Number::from(10)))
        .custom("enable_code_execution", serde_json::Value::Bool(true))
        .build()?;
    
    println!("   框架: {}", config.framework);
    println!("   最大Agent数: {:?}", config.custom.get("max_agents"));
    println!("   对话轮数: {:?}", config.custom.get("conversation_rounds"));
    
    // 4. 构建AutoGen插件
    println!("\n🔧 4. 构建AutoGen插件");
    let plugin = PluginBuilder::new()
        .framework("autogen")
        .config(config)
        .capability(PluginCapability::MultiAgentConversation)
        .capability(PluginCapability::CodeGeneration)
        .capability(PluginCapability::TextProcessing)
        .capability(PluginCapability::ToolCalling)
        .build()
        .await?;
    
    println!("   插件ID: {}", plugin.get_info().metadata.id);
    println!("   插件名称: {}", plugin.get_info().metadata.name);
    println!("   支持能力: {:?}", plugin.get_capabilities());
    
    // 5. 创建AutoGen框架实例
    println!("\n🏗️  5. 创建AutoGen框架实例");
    let framework_config = agentx_sdk::framework::FrameworkConfig {
        framework_type: FrameworkType::AutoGen,
        runtime_path: "python".to_string(),
        working_directory: "./examples/autogen".to_string(),
        environment_variables: {
            let mut env = HashMap::new();
            env.insert("PYTHONPATH".to_string(), ".".to_string());
            env.insert("OPENAI_API_KEY".to_string(), "your-api-key-here".to_string());
            env
        },
        startup_args: vec![
            "-c".to_string(),
            "import autogen; print('AutoGen loaded successfully')".to_string(),
        ],
        dependencies: vec![
            "pyautogen".to_string(),
            "openai".to_string(),
        ],
        custom_config: HashMap::new(),
    };
    
    let mut framework = FrameworkBuilder::new()
        .framework_type(FrameworkType::AutoGen)
        .config(framework_config)
        .build()
        .await?;
    
    println!("   框架类型: {:?}", framework.get_type());
    println!("   运行状态: {}", framework.is_running());
    
    // 6. 演示多Agent对话场景
    println!("\n💬 6. 演示多Agent对话场景");
    
    // 创建多Agent对话任务
    let conversation_scenarios = vec![
        ("软件开发团队", "设计一个简单的待办事项应用"),
        ("研究团队", "分析人工智能的发展趋势"),
        ("创意团队", "为新产品想一个营销策略"),
    ];
    
    for (team_type, task) in conversation_scenarios {
        println!("\n   👥 场景: {} - {}", team_type, task);
        
        let conversation_message = A2AMessage::new_data(
            MessageRole::User,
            serde_json::json!({
                "type": "multi_agent_conversation",
                "team_type": team_type,
                "task": task,
                "agents": [
                    {
                        "name": "project_manager",
                        "role": "项目经理",
                        "description": "负责协调和管理项目进度"
                    },
                    {
                        "name": "developer",
                        "role": "开发者",
                        "description": "负责技术实现和代码编写"
                    },
                    {
                        "name": "designer",
                        "role": "设计师",
                        "description": "负责用户界面和用户体验设计"
                    }
                ],
                "max_rounds": 5
            })
        );
        
        if env_info.available {
            match framework.process_message(conversation_message).await {
                Ok(Some(response)) => {
                    println!("     ✅ 对话结果: {}", 
                        agentx_sdk::MessageUtils::extract_text_content(&response));
                },
                Ok(None) => {
                    println!("     ℹ️  对话无结果");
                },
                Err(e) => {
                    println!("     ❌ 对话失败: {:?}", e);
                }
            }
        } else {
            println!("     🔄 模拟对话结果:");
            println!("       项目经理: 我们需要明确需求和时间线");
            println!("       开发者: 建议使用React和Node.js技术栈");
            println!("       设计师: 界面应该简洁直观，符合用户习惯");
            println!("       项目经理: 很好，让我们制定详细的开发计划");
        }
    }
    
    // 7. 演示代码生成功能
    println!("\n💻 7. 演示代码生成功能");
    
    let code_generation_tasks = vec![
        "创建一个Python函数来计算斐波那契数列",
        "编写一个JavaScript函数来验证邮箱格式",
        "生成一个SQL查询来统计用户活跃度",
    ];
    
    for task in code_generation_tasks {
        println!("\n   📝 代码生成任务: {}", task);
        
        let code_message = A2AMessage::new_data(
            MessageRole::User,
            serde_json::json!({
                "type": "code_generation",
                "task": task,
                "language": "auto_detect",
                "include_tests": true,
                "include_documentation": true
            })
        );
        
        if env_info.available {
            match framework.process_message(code_message).await {
                Ok(Some(response)) => {
                    println!("     ✅ 生成的代码: {}", 
                        agentx_sdk::MessageUtils::extract_text_content(&response));
                },
                Ok(None) => {
                    println!("     ℹ️  代码生成无结果");
                },
                Err(e) => {
                    println!("     ❌ 代码生成失败: {:?}", e);
                }
            }
        } else {
            println!("     🔄 模拟生成的代码:");
            match task {
                t if t.contains("斐波那契") => {
                    println!("       def fibonacci(n):");
                    println!("           if n <= 1: return n");
                    println!("           return fibonacci(n-1) + fibonacci(n-2)");
                },
                t if t.contains("邮箱") => {
                    println!("       function validateEmail(email) {{");
                    println!("           const re = /^[^\\s@]+@[^\\s@]+\\.[^\\s@]+$/;");
                    println!("           return re.test(email);");
                    println!("       }}");
                },
                t if t.contains("SQL") => {
                    println!("       SELECT user_id, COUNT(*) as activity_count");
                    println!("       FROM user_activities");
                    println!("       WHERE created_at >= DATE_SUB(NOW(), INTERVAL 30 DAY)");
                    println!("       GROUP BY user_id ORDER BY activity_count DESC;");
                },
                _ => {
                    println!("       // 生成的代码将在这里显示");
                }
            }
        }
    }
    
    // 8. 演示Agent协作工作流
    println!("\n🔄 8. 演示Agent协作工作流");
    
    let workflow_message = A2AMessage::new_data(
        MessageRole::User,
        serde_json::json!({
            "type": "agent_workflow",
            "workflow_name": "product_development",
            "steps": [
                {
                    "step": "requirement_analysis",
                    "agent": "business_analyst",
                    "description": "分析产品需求"
                },
                {
                    "step": "technical_design",
                    "agent": "architect",
                    "description": "设计技术架构"
                },
                {
                    "step": "implementation",
                    "agent": "developer",
                    "description": "实现核心功能"
                },
                {
                    "step": "testing",
                    "agent": "qa_engineer",
                    "description": "质量保证测试"
                },
                {
                    "step": "deployment",
                    "agent": "devops_engineer",
                    "description": "部署到生产环境"
                }
            ]
        })
    );
    
    println!("   🔄 启动产品开发工作流...");
    
    if env_info.available {
        match framework.process_message(workflow_message).await {
            Ok(Some(response)) => {
                println!("     ✅ 工作流执行结果: {}", 
                    agentx_sdk::MessageUtils::extract_text_content(&response));
            },
            Ok(None) => {
                println!("     ℹ️  工作流无结果");
            },
            Err(e) => {
                println!("     ❌ 工作流执行失败: {:?}", e);
            }
        }
    } else {
        println!("     🔄 模拟工作流执行:");
        println!("       ✅ 需求分析完成 - 业务分析师");
        println!("       ✅ 技术设计完成 - 架构师");
        println!("       ✅ 功能实现完成 - 开发者");
        println!("       ✅ 质量测试完成 - 测试工程师");
        println!("       ✅ 生产部署完成 - 运维工程师");
    }
    
    // 9. 演示插件统计
    println!("\n📊 9. 插件统计信息");
    let stats = plugin.get_stats();
    println!("   处理消息数: {}", stats.messages_processed);
    println!("   多Agent对话数: 3 (模拟)");
    println!("   代码生成任务数: 3 (模拟)");
    println!("   工作流执行数: 1 (模拟)");
    println!("   平均响应时间: {:.2}ms", stats.avg_response_time_ms);
    
    // 10. 清理资源
    println!("\n🧹 10. 清理资源");
    
    if framework.is_running() {
        framework.stop().await?;
        println!("   ✅ AutoGen框架已停止");
    }
    
    println!("   ✅ 资源清理完成");
    
    // 11. 总结
    println!("\n📋 11. 演示总结");
    println!("   ✅ SDK初始化成功");
    println!("   ✅ Python环境检测完成");
    println!("   ✅ AutoGen配置创建成功");
    println!("   ✅ 插件构建成功");
    println!("   ✅ 框架实例创建成功");
    println!("   ✅ 多Agent对话演示完成");
    println!("   ✅ 代码生成演示完成");
    println!("   ✅ 工作流协作演示完成");
    println!("   ✅ 统计信息查看完成");
    println!("   ✅ 资源清理完成");
    
    println!("\n🎉 AutoGen插件演示完成！");
    println!("===========================");
    
    Ok(())
}

/// 创建示例AutoGen脚本
async fn create_example_script() -> A2AResult<()> {
    let script_content = r#"
#!/usr/bin/env python3
"""
AutoGen AgentX 插件示例脚本
"""

import sys
import json
import autogen

def create_agents():
    """创建AutoGen代理"""
    config_list = [
        {
            "model": "gpt-3.5-turbo",
            "api_key": "your-api-key-here"
        }
    ]
    
    # 创建用户代理
    user_proxy = autogen.UserProxyAgent(
        name="user_proxy",
        system_message="A human admin.",
        code_execution_config={"last_n_messages": 2, "work_dir": "groupchat"},
        human_input_mode="NEVER"
    )
    
    # 创建助手代理
    assistant = autogen.AssistantAgent(
        name="assistant",
        llm_config={"config_list": config_list},
        system_message="You are a helpful AI assistant."
    )
    
    return user_proxy, assistant

def process_conversation(message_data):
    """处理多Agent对话"""
    try:
        message = json.loads(message_data)
        task = message.get('task', '')
        
        user_proxy, assistant = create_agents()
        
        # 启动对话
        user_proxy.initiate_chat(
            assistant,
            message=task,
            max_turns=5
        )
        
        return {
            "type": "conversation_result",
            "status": "success",
            "summary": "Multi-agent conversation completed successfully"
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
        result = process_conversation(message_data)
        print(json.dumps(result))
    else:
        print("AutoGen AgentX Plugin Ready")
"#;
    
    // 创建示例目录
    tokio::fs::create_dir_all("./examples/autogen").await
        .map_err(|e| agentx_sdk::A2AError::internal(format!("创建目录失败: {}", e)))?;
    
    // 写入脚本文件
    tokio::fs::write("./examples/autogen/plugin.py", script_content).await
        .map_err(|e| agentx_sdk::A2AError::internal(format!("写入脚本失败: {}", e)))?;
    
    println!("   ✅ 示例脚本已创建: ./examples/autogen/plugin.py");
    
    Ok(())
}
