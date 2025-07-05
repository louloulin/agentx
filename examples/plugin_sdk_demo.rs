//! AgentX SDK综合演示
//! 
//! 展示多框架插件的统一管理和协作

use agentx_sdk::{
    init_sdk, quick_start, create_server, create_client,
    PluginBuilder, PluginCapability, FrameworkUtils, PluginUtils,
    A2AMessage, MessageRole, SDK_VERSION, SUPPORTED_FRAMEWORKS,
};
use std::collections::HashMap;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 AgentX SDK综合演示");
    println!("======================");
    println!("SDK版本: {}", SDK_VERSION);
    println!("支持的框架: {:?}", SUPPORTED_FRAMEWORKS);
    
    // 1. 初始化SDK
    println!("\n📦 1. 初始化AgentX SDK");
    init_sdk().await?;
    
    // 2. 环境检测
    println!("\n🔍 2. 检测多框架环境");
    let frameworks_to_check = vec!["langchain", "autogen", "mastra"];
    let mut available_frameworks = Vec::new();
    
    for framework in frameworks_to_check {
        match FrameworkUtils::detect_framework_environment(framework).await {
            Ok(env_info) => {
                println!("   {} - {}: {} ({})", 
                    if env_info.available { "✅" } else { "❌" },
                    framework, 
                    env_info.version,
                    if env_info.available { "可用" } else { "不可用" }
                );
                if env_info.available {
                    available_frameworks.push(framework);
                }
            },
            Err(_) => {
                println!("   ❌ {} - 检测失败", framework);
            }
        }
    }
    
    println!("   可用框架数量: {}", available_frameworks.len());
    
    // 3. 快速启动演示
    println!("\n⚡ 3. 快速启动演示");
    
    if !available_frameworks.is_empty() {
        let framework = available_frameworks[0];
        println!("   使用框架: {}", framework);
        
        match quick_start(framework, None).await {
            Ok(plugin) => {
                println!("   ✅ 快速启动成功");
                println!("     插件ID: {}", plugin.get_info().metadata.id);
                println!("     插件状态: {:?}", plugin.get_status());
            },
            Err(e) => {
                println!("   ❌ 快速启动失败: {:?}", e);
            }
        }
    } else {
        println!("   ⚠️  没有可用的框架，跳过快速启动演示");
    }
    
    // 4. 创建多框架插件
    println!("\n🔧 4. 创建多框架插件");
    let mut plugins = HashMap::new();
    
    for framework in &["langchain", "autogen", "mastra"] {
        println!("   创建{}插件...", framework);
        
        match PluginBuilder::new()
            .framework(framework)
            .capability(PluginCapability::TextProcessing)
            .capability(PluginCapability::ToolCalling)
            .build()
            .await
        {
            Ok(plugin) => {
                println!("     ✅ {}插件创建成功", framework);
                plugins.insert(framework.to_string(), plugin);
            },
            Err(e) => {
                println!("     ❌ {}插件创建失败: {:?}", framework, e);
            }
        }
    }
    
    println!("   成功创建插件数量: {}", plugins.len());
    
    // 5. 演示插件协作
    println!("\n🤝 5. 演示插件协作");
    
    let collaboration_scenarios = vec![
        ("文档生成", "LangChain生成内容，AutoGen优化，Mastra格式化"),
        ("数据分析", "Mastra处理数据，LangChain分析，AutoGen生成报告"),
        ("代码审查", "AutoGen生成代码，LangChain审查，Mastra部署"),
    ];
    
    for (scenario, description) in collaboration_scenarios {
        println!("\n   📋 协作场景: {}", scenario);
        println!("     描述: {}", description);
        
        let collaboration_message = A2AMessage::new_data(
            MessageRole::User,
            serde_json::json!({
                "type": "collaboration",
                "scenario": scenario,
                "description": description,
                "participants": ["langchain", "autogen", "mastra"],
                "workflow": [
                    {"step": 1, "framework": "langchain", "action": "generate"},
                    {"step": 2, "framework": "autogen", "action": "optimize"},
                    {"step": 3, "framework": "mastra", "action": "format"}
                ]
            })
        );
        
        // 模拟协作流程
        for framework in &["langchain", "autogen", "mastra"] {
            if let Some(plugin) = plugins.get_mut(*framework) {
                match plugin.process_message(collaboration_message.clone()).await {
                    Ok(Some(response)) => {
                        println!("     ✅ {} 处理完成: {}", 
                            framework, 
                            agentx_sdk::MessageUtils::extract_text_content(&response)[..50.min(agentx_sdk::MessageUtils::extract_text_content(&response).len())].to_string() + "..."
                        );
                    },
                    Ok(None) => {
                        println!("     ℹ️  {} 无响应", framework);
                    },
                    Err(e) => {
                        println!("     ❌ {} 处理失败: {:?}", framework, e);
                    }
                }
            } else {
                println!("     🔄 {} 模拟处理完成", framework);
            }
        }
        
        println!("     ✅ 协作场景完成");
    }
    
    // 6. 服务器和客户端演示
    println!("\n🌐 6. 服务器和客户端演示");
    
    // 创建服务器
    println!("   创建插件服务器...");
    match create_server("127.0.0.1:50052").await {
        Ok(mut server) => {
            println!("     ✅ 服务器创建成功");
            
            // 启动服务器（模拟）
            match server.start().await {
                Ok(_) => {
                    println!("     ✅ 服务器启动成功");
                    
                    // 创建客户端
                    println!("   创建插件客户端...");
                    match create_client("http://127.0.0.1:50052").await {
                        Ok(client) => {
                            println!("     ✅ 客户端创建成功");
                            
                            // 测试客户端连接
                            let test_message = A2AMessage::agent_message("Hello from client".to_string());
                            match client.send_message(test_message).await {
                                Ok(response) => {
                                    if let Some(resp) = response {
                                        println!("     ✅ 客户端通信成功: {}", 
                                            agentx_sdk::MessageUtils::extract_text_content(&resp));
                                    } else {
                                        println!("     ℹ️  客户端通信无响应");
                                    }
                                },
                                Err(e) => {
                                    println!("     ❌ 客户端通信失败: {:?}", e);
                                }
                            }
                        },
                        Err(e) => {
                            println!("     ❌ 客户端创建失败: {:?}", e);
                        }
                    }
                    
                    // 停止服务器
                    let _ = server.stop().await;
                    println!("     ✅ 服务器已停止");
                },
                Err(e) => {
                    println!("     ❌ 服务器启动失败: {:?}", e);
                }
            }
        },
        Err(e) => {
            println!("     ❌ 服务器创建失败: {:?}", e);
        }
    }
    
    // 7. 性能基准测试
    println!("\n⚡ 7. 性能基准测试");
    
    let benchmark_message = A2AMessage::agent_message("Benchmark test message".to_string());
    let message_count = 100;
    
    for framework in &["langchain", "autogen", "mastra"] {
        if let Some(plugin) = plugins.get_mut(*framework) {
            println!("   测试{}性能...", framework);
            
            let start_time = std::time::Instant::now();
            let mut success_count = 0;
            
            for _ in 0..message_count {
                match plugin.process_message(benchmark_message.clone()).await {
                    Ok(_) => success_count += 1,
                    Err(_) => {}
                }
            }
            
            let duration = start_time.elapsed();
            let throughput = (success_count as f64) / duration.as_secs_f64();
            let avg_latency = duration.as_millis() as f64 / success_count as f64;
            
            println!("     ✅ {}性能结果:", framework);
            println!("       成功处理: {}/{}", success_count, message_count);
            println!("       吞吐量: {:.0} 消息/秒", throughput);
            println!("       平均延迟: {:.2}ms", avg_latency);
        }
    }
    
    // 8. 工具和实用功能演示
    println!("\n🛠️  8. 工具和实用功能演示");
    
    // 插件版本比较
    println!("   版本比较测试:");
    let versions = vec![("1.0.0", "1.0.1"), ("2.1.0", "2.0.5"), ("1.0.0", "1.0.0")];
    for (v1, v2) in versions {
        let comparison = PluginUtils::compare_versions(v1, v2);
        println!("     {} vs {} = {:?}", v1, v2, comparison);
    }
    
    // 消息验证
    println!("   消息验证测试:");
    let test_messages = vec![
        A2AMessage::agent_message("Valid message".to_string()),
        A2AMessage::agent_message("".to_string()), // 空消息
    ];
    
    for (i, message) in test_messages.iter().enumerate() {
        match agentx_sdk::MessageUtils::validate_message(message) {
            Ok(_) => println!("     消息{}: ✅ 验证通过", i + 1),
            Err(e) => println!("     消息{}: ❌ 验证失败 - {:?}", i + 1, e),
        }
    }
    
    // 配置管理
    println!("   配置管理测试:");
    let env_config = agentx_sdk::ConfigUtils::load_from_env();
    println!("     环境配置框架: {}", env_config.framework);
    
    // 9. 统计信息汇总
    println!("\n📊 9. 统计信息汇总");
    
    let mut total_messages = 0;
    let mut total_errors = 0;
    
    for (framework, plugin) in &plugins {
        let stats = plugin.get_stats();
        println!("   {} 统计:", framework);
        println!("     处理消息数: {}", stats.messages_processed);
        println!("     错误数: {}", stats.errors);
        println!("     平均响应时间: {:.2}ms", stats.avg_response_time_ms);
        
        total_messages += stats.messages_processed;
        total_errors += stats.errors;
    }
    
    println!("   总计:");
    println!("     总消息数: {}", total_messages);
    println!("     总错误数: {}", total_errors);
    println!("     成功率: {:.1}%", 
        if total_messages > 0 { 
            ((total_messages - total_errors) as f64 / total_messages as f64) * 100.0 
        } else { 
            0.0 
        }
    );
    
    // 10. 清理资源
    println!("\n🧹 10. 清理资源");
    
    for (framework, mut plugin) in plugins {
        match plugin.stop().await {
            Ok(_) => println!("   ✅ {}插件已停止", framework),
            Err(e) => println!("   ❌ {}插件停止失败: {:?}", framework, e),
        }
    }
    
    println!("   ✅ 所有资源清理完成");
    
    // 11. 总结
    println!("\n📋 11. 演示总结");
    println!("   ✅ SDK初始化成功");
    println!("   ✅ 多框架环境检测完成");
    println!("   ✅ 快速启动演示完成");
    println!("   ✅ 多框架插件创建完成");
    println!("   ✅ 插件协作演示完成");
    println!("   ✅ 服务器客户端演示完成");
    println!("   ✅ 性能基准测试完成");
    println!("   ✅ 工具功能演示完成");
    println!("   ✅ 统计信息汇总完成");
    println!("   ✅ 资源清理完成");
    
    println!("\n🎉 AgentX SDK综合演示完成！");
    println!("=============================");
    println!("AgentX SDK为多框架AI Agent开发提供了统一、高效的解决方案！");
    
    Ok(())
}
