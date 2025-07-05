//! Mastra插件演示
//! 
//! 展示如何使用AgentX SDK创建和管理Mastra (Node.js/TypeScript) 插件

use agentx_sdk::{
    init_sdk, PluginBuilder, FrameworkBuilder, ConfigBuilder,
    FrameworkType, PluginCapability, FrameworkUtils,
    A2AMessage, MessageRole, A2AResult,
};
use std::collections::HashMap;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🟢 Mastra (Node.js) 插件演示");
    println!("=============================");
    
    // 1. 初始化SDK
    println!("\n📦 1. 初始化AgentX SDK");
    init_sdk().await?;
    
    // 2. 检测Node.js环境
    println!("\n🔍 2. 检测Node.js环境");
    let env_info = FrameworkUtils::detect_framework_environment("mastra").await?;
    println!("   运行时: {}", env_info.runtime);
    println!("   版本: {}", env_info.version);
    println!("   可用: {}", env_info.available);
    
    if !env_info.available {
        println!("   ⚠️  Node.js环境不可用，演示将使用模拟模式");
    }
    
    // 3. 创建Mastra配置
    println!("\n⚙️  3. 创建Mastra配置");
    let config = ConfigBuilder::new()
        .framework("mastra")
        .runtime_path("node")
        .working_directory("./examples/mastra")
        .env_var("NODE_ENV", "development")
        .env_var("OPENAI_API_KEY", "your-api-key-here")
        .custom("typescript", serde_json::Value::Bool(true))
        .custom("hot_reload", serde_json::Value::Bool(true))
        .custom("max_memory", serde_json::Value::String("512M".to_string()))
        .build()?;
    
    println!("   框架: {}", config.framework);
    println!("   TypeScript支持: {:?}", config.custom.get("typescript"));
    println!("   热重载: {:?}", config.custom.get("hot_reload"));
    
    // 4. 构建Mastra插件
    println!("\n🔧 4. 构建Mastra插件");
    let plugin = PluginBuilder::new()
        .framework("mastra")
        .config(config)
        .capability(PluginCapability::TextProcessing)
        .capability(PluginCapability::WorkflowExecution)
        .capability(PluginCapability::ToolCalling)
        .capability(PluginCapability::DataAnalysis)
        .build()
        .await?;
    
    println!("   插件ID: {}", plugin.get_info().metadata.id);
    println!("   插件名称: {}", plugin.get_info().metadata.name);
    println!("   支持能力: {:?}", plugin.get_capabilities());
    
    // 5. 创建Mastra框架实例
    println!("\n🏗️  5. 创建Mastra框架实例");
    let framework_config = agentx_sdk::framework::FrameworkConfig {
        framework_type: FrameworkType::Mastra,
        runtime_path: "node".to_string(),
        working_directory: "./examples/mastra".to_string(),
        environment_variables: {
            let mut env = HashMap::new();
            env.insert("NODE_ENV".to_string(), "development".to_string());
            env.insert("OPENAI_API_KEY".to_string(), "your-api-key-here".to_string());
            env
        },
        startup_args: vec![
            "-e".to_string(),
            "console.log('Mastra framework loaded successfully')".to_string(),
        ],
        dependencies: vec![
            "@mastra/core".to_string(),
            "@mastra/engine".to_string(),
            "typescript".to_string(),
        ],
        custom_config: HashMap::new(),
    };
    
    let mut framework = FrameworkBuilder::new()
        .framework_type(FrameworkType::Mastra)
        .config(framework_config)
        .build()
        .await?;
    
    println!("   框架类型: {:?}", framework.get_type());
    println!("   运行状态: {}", framework.is_running());
    
    // 6. 演示Mastra工作流
    println!("\n🔄 6. 演示Mastra工作流");
    
    let workflow_scenarios = vec![
        ("数据处理流水线", "处理用户上传的CSV文件并生成报告"),
        ("内容生成工作流", "根据用户输入生成博客文章"),
        ("API集成流程", "从多个API获取数据并合并处理"),
    ];
    
    for (workflow_name, description) in workflow_scenarios {
        println!("\n   🔄 工作流: {} - {}", workflow_name, description);
        
        let workflow_message = A2AMessage::new_data(
            MessageRole::User,
            serde_json::json!({
                "type": "mastra_workflow",
                "workflow_name": workflow_name,
                "description": description,
                "steps": [
                    {
                        "id": "input_validation",
                        "type": "validator",
                        "config": {
                            "schema": "user_input_schema"
                        }
                    },
                    {
                        "id": "data_processing",
                        "type": "processor",
                        "config": {
                            "engine": "mastra_engine"
                        }
                    },
                    {
                        "id": "output_generation",
                        "type": "generator",
                        "config": {
                            "format": "json"
                        }
                    }
                ],
                "triggers": ["user_request", "scheduled"],
                "outputs": ["processed_data", "report"]
            })
        );
        
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
            println!("       ✅ 输入验证完成");
            println!("       ✅ 数据处理完成");
            println!("       ✅ 输出生成完成");
            println!("       📊 工作流执行成功");
        }
    }
    
    // 7. 演示Mastra工具集成
    println!("\n🛠️  7. 演示Mastra工具集成");
    
    let tool_integrations = vec![
        ("OpenAI GPT", "文本生成和对话"),
        ("Google Sheets", "数据读写操作"),
        ("Slack API", "消息发送和通知"),
        ("GitHub API", "代码仓库管理"),
    ];
    
    for (tool_name, description) in tool_integrations {
        println!("\n   🔧 工具集成: {} - {}", tool_name, description);
        
        let tool_message = A2AMessage::new_data(
            MessageRole::User,
            serde_json::json!({
                "type": "tool_integration",
                "tool_name": tool_name,
                "description": description,
                "action": "execute",
                "parameters": {
                    "input": "test data",
                    "options": {
                        "timeout": 30000,
                        "retry": 3
                    }
                }
            })
        );
        
        if env_info.available {
            match framework.process_message(tool_message).await {
                Ok(Some(response)) => {
                    println!("     ✅ 工具执行结果: {}", 
                        agentx_sdk::MessageUtils::extract_text_content(&response));
                },
                Ok(None) => {
                    println!("     ℹ️  工具执行无结果");
                },
                Err(e) => {
                    println!("     ❌ 工具执行失败: {:?}", e);
                }
            }
        } else {
            println!("     🔄 模拟工具执行:");
            match tool_name {
                "OpenAI GPT" => println!("       📝 生成文本: 'Hello from Mastra!'"),
                "Google Sheets" => println!("       📊 数据操作: 读取了10行数据"),
                "Slack API" => println!("       💬 消息发送: 通知已发送到#general频道"),
                "GitHub API" => println!("       🔧 仓库操作: 获取了最新的提交信息"),
                _ => println!("       ✅ 工具执行完成"),
            }
        }
    }
    
    // 8. 演示TypeScript类型安全
    println!("\n🔒 8. 演示TypeScript类型安全");
    
    let typescript_message = A2AMessage::new_data(
        MessageRole::User,
        serde_json::json!({
            "type": "typescript_validation",
            "code": r#"
                interface UserData {
                    id: number;
                    name: string;
                    email: string;
                    active: boolean;
                }
                
                function processUser(user: UserData): string {
                    return `Processing user: ${user.name} (${user.email})`;
                }
                
                const testUser: UserData = {
                    id: 1,
                    name: "John Doe",
                    email: "john@example.com",
                    active: true
                };
                
                console.log(processUser(testUser));
            "#,
            "validate_types": true,
            "compile": true
        })
    );
    
    println!("   🔍 TypeScript代码验证...");
    
    if env_info.available {
        match framework.process_message(typescript_message).await {
            Ok(Some(response)) => {
                println!("     ✅ TypeScript验证结果: {}", 
                    agentx_sdk::MessageUtils::extract_text_content(&response));
            },
            Ok(None) => {
                println!("     ℹ️  TypeScript验证无结果");
            },
            Err(e) => {
                println!("     ❌ TypeScript验证失败: {:?}", e);
            }
        }
    } else {
        println!("     🔄 模拟TypeScript验证:");
        println!("       ✅ 类型检查通过");
        println!("       ✅ 编译成功");
        println!("       📝 输出: Processing user: John Doe (john@example.com)");
    }
    
    // 9. 演示实时数据处理
    println!("\n📊 9. 演示实时数据处理");
    
    let realtime_message = A2AMessage::new_data(
        MessageRole::User,
        serde_json::json!({
            "type": "realtime_processing",
            "data_source": "websocket_stream",
            "processing_config": {
                "batch_size": 100,
                "window_size": "5s",
                "aggregation": "sum",
                "filters": [
                    {"field": "status", "value": "active"},
                    {"field": "amount", "operator": ">", "value": 0}
                ]
            },
            "output_format": "json",
            "real_time": true
        })
    );
    
    println!("   📡 启动实时数据处理...");
    
    if env_info.available {
        match framework.process_message(realtime_message).await {
            Ok(Some(response)) => {
                println!("     ✅ 实时处理结果: {}", 
                    agentx_sdk::MessageUtils::extract_text_content(&response));
            },
            Ok(None) => {
                println!("     ℹ️  实时处理无结果");
            },
            Err(e) => {
                println!("     ❌ 实时处理失败: {:?}", e);
            }
        }
    } else {
        println!("     🔄 模拟实时处理:");
        println!("       📊 处理了500条数据记录");
        println!("       🔍 应用了2个过滤器");
        println!("       📈 聚合结果: 总和 = 12,345");
        println!("       ⏱️  处理延迟: 15ms");
    }
    
    // 10. 演示插件统计
    println!("\n📊 10. 插件统计信息");
    let stats = plugin.get_stats();
    println!("   处理消息数: {}", stats.messages_processed);
    println!("   工作流执行数: 3 (模拟)");
    println!("   工具集成数: 4 (模拟)");
    println!("   TypeScript验证数: 1 (模拟)");
    println!("   实时处理任务数: 1 (模拟)");
    println!("   平均响应时间: {:.2}ms", stats.avg_response_time_ms);
    
    // 11. 清理资源
    println!("\n🧹 11. 清理资源");
    
    if framework.is_running() {
        framework.stop().await?;
        println!("   ✅ Mastra框架已停止");
    }
    
    println!("   ✅ 资源清理完成");
    
    // 12. 总结
    println!("\n📋 12. 演示总结");
    println!("   ✅ SDK初始化成功");
    println!("   ✅ Node.js环境检测完成");
    println!("   ✅ Mastra配置创建成功");
    println!("   ✅ 插件构建成功");
    println!("   ✅ 框架实例创建成功");
    println!("   ✅ 工作流演示完成");
    println!("   ✅ 工具集成演示完成");
    println!("   ✅ TypeScript验证演示完成");
    println!("   ✅ 实时处理演示完成");
    println!("   ✅ 统计信息查看完成");
    println!("   ✅ 资源清理完成");
    
    println!("\n🎉 Mastra插件演示完成！");
    println!("============================");
    
    Ok(())
}

/// 创建示例Mastra脚本
async fn create_example_script() -> A2AResult<()> {
    let package_json = r#"
{
  "name": "mastra-agentx-plugin",
  "version": "1.0.0",
  "description": "Mastra AgentX Plugin Example",
  "main": "index.js",
  "scripts": {
    "start": "node index.js",
    "dev": "nodemon index.js",
    "build": "tsc",
    "test": "jest"
  },
  "dependencies": {
    "@mastra/core": "^0.1.0",
    "@mastra/engine": "^0.1.0",
    "typescript": "^5.0.0"
  },
  "devDependencies": {
    "@types/node": "^20.0.0",
    "nodemon": "^3.0.0",
    "jest": "^29.0.0"
  }
}
"#;
    
    let typescript_code = r#"
import { Mastra, Workflow, Tool } from '@mastra/core';

interface MessageData {
    type: string;
    content: string;
    metadata?: Record<string, any>;
}

class MastraAgentXPlugin {
    private mastra: Mastra;
    
    constructor() {
        this.mastra = new Mastra({
            name: 'AgentX Plugin',
            version: '1.0.0'
        });
        
        this.setupWorkflows();
        this.setupTools();
    }
    
    private setupWorkflows(): void {
        // 设置数据处理工作流
        const dataProcessingWorkflow = new Workflow({
            name: 'data_processing',
            steps: [
                {
                    id: 'validate',
                    type: 'validator',
                    config: { schema: 'input_schema' }
                },
                {
                    id: 'process',
                    type: 'processor',
                    config: { engine: 'mastra_engine' }
                },
                {
                    id: 'output',
                    type: 'generator',
                    config: { format: 'json' }
                }
            ]
        });
        
        this.mastra.addWorkflow(dataProcessingWorkflow);
    }
    
    private setupTools(): void {
        // 设置OpenAI工具
        const openaiTool = new Tool({
            name: 'openai_gpt',
            description: 'OpenAI GPT text generation',
            execute: async (input: string) => {
                // 模拟OpenAI调用
                return `Generated response for: ${input}`;
            }
        });
        
        this.mastra.addTool(openaiTool);
    }
    
    async processMessage(messageData: string): Promise<any> {
        try {
            const message: MessageData = JSON.parse(messageData);
            
            switch (message.type) {
                case 'mastra_workflow':
                    return await this.executeWorkflow(message);
                case 'tool_integration':
                    return await this.executeTool(message);
                case 'typescript_validation':
                    return await this.validateTypeScript(message);
                default:
                    return {
                        type: 'response',
                        content: `Processed message of type: ${message.type}`,
                        status: 'success'
                    };
            }
        } catch (error) {
            return {
                type: 'error',
                content: error.message,
                status: 'error'
            };
        }
    }
    
    private async executeWorkflow(message: MessageData): Promise<any> {
        // 执行Mastra工作流
        const result = await this.mastra.executeWorkflow('data_processing', message.content);
        return {
            type: 'workflow_result',
            content: result,
            status: 'success'
        };
    }
    
    private async executeTool(message: MessageData): Promise<any> {
        // 执行工具
        const toolName = message.metadata?.tool_name || 'openai_gpt';
        const result = await this.mastra.executeTool(toolName, message.content);
        return {
            type: 'tool_result',
            content: result,
            status: 'success'
        };
    }
    
    private async validateTypeScript(message: MessageData): Promise<any> {
        // TypeScript验证
        return {
            type: 'typescript_result',
            content: 'TypeScript validation completed',
            status: 'success'
        };
    }
}

// 主程序
const plugin = new MastraAgentXPlugin();

if (process.argv.length > 2) {
    const messageData = process.argv[2];
    plugin.processMessage(messageData)
        .then(result => console.log(JSON.stringify(result)))
        .catch(error => console.error(JSON.stringify({
            type: 'error',
            content: error.message,
            status: 'error'
        })));
} else {
    console.log('Mastra AgentX Plugin Ready');
}
"#;
    
    // 创建示例目录
    tokio::fs::create_dir_all("./examples/mastra").await
        .map_err(|e| agentx_sdk::A2AError::internal(format!("创建目录失败: {}", e)))?;
    
    // 写入package.json
    tokio::fs::write("./examples/mastra/package.json", package_json).await
        .map_err(|e| agentx_sdk::A2AError::internal(format!("写入package.json失败: {}", e)))?;
    
    // 写入TypeScript代码
    tokio::fs::write("./examples/mastra/index.ts", typescript_code).await
        .map_err(|e| agentx_sdk::A2AError::internal(format!("写入TypeScript代码失败: {}", e)))?;
    
    println!("   ✅ 示例文件已创建:");
    println!("     - ./examples/mastra/package.json");
    println!("     - ./examples/mastra/index.ts");
    
    Ok(())
}
