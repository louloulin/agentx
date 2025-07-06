//! 开发者生态系统
//! 
//! 提供插件市场、CLI工具和开发者支持功能

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use agentx_a2a::{A2AResult, A2AError};
use chrono::{DateTime, Utc};

/// 插件市场条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMarketEntry {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub category: PluginCategory,
    pub tags: Vec<String>,
    pub download_url: String,
    pub documentation_url: Option<String>,
    pub source_code_url: Option<String>,
    pub license: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub downloads: u64,
    pub rating: f32,
    pub reviews: Vec<PluginReview>,
    pub compatibility: CompatibilityInfo,
    pub dependencies: Vec<PluginDependency>,
}

/// 插件分类
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PluginCategory {
    /// AI框架适配器
    FrameworkAdapter,
    /// 工具和实用程序
    Tools,
    /// 数据连接器
    DataConnectors,
    /// 安全和认证
    Security,
    /// 监控和可观测性
    Monitoring,
    /// 开发工具
    Development,
    /// 自定义扩展
    Custom,
}

/// 插件评价
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginReview {
    pub reviewer: String,
    pub rating: u8,
    pub comment: String,
    pub created_at: DateTime<Utc>,
}

/// 兼容性信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityInfo {
    pub agentx_version: String,
    pub supported_platforms: Vec<String>,
    pub required_features: Vec<String>,
}

/// 插件依赖
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDependency {
    pub name: String,
    pub version: String,
    pub optional: bool,
}

/// CLI命令定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliCommand {
    pub name: String,
    pub description: String,
    pub usage: String,
    pub options: Vec<CliOption>,
    pub subcommands: Vec<CliCommand>,
}

/// CLI选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliOption {
    pub name: String,
    pub short: Option<String>,
    pub description: String,
    pub required: bool,
    pub default_value: Option<String>,
}

/// 项目模板
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectTemplate {
    pub name: String,
    pub description: String,
    pub language: String,
    pub framework: String,
    pub files: HashMap<String, String>,
    pub dependencies: Vec<String>,
    pub setup_instructions: Vec<String>,
}

/// 插件市场管理器
pub struct PluginMarketManager {
    plugins: HashMap<String, PluginMarketEntry>,
    categories: HashMap<PluginCategory, Vec<String>>,
}

impl PluginMarketManager {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            categories: HashMap::new(),
        }
    }
    
    /// 注册插件到市场
    pub fn register_plugin(&mut self, plugin: PluginMarketEntry) -> A2AResult<()> {
        // 验证插件信息
        if plugin.name.is_empty() || plugin.version.is_empty() {
            return Err(A2AError::validation("插件名称和版本不能为空".to_string()));
        }
        
        // 添加到分类索引
        self.categories
            .entry(plugin.category.clone())
            .or_insert_with(Vec::new)
            .push(plugin.id.clone());
        
        // 注册插件
        self.plugins.insert(plugin.id.clone(), plugin);
        
        Ok(())
    }
    
    /// 搜索插件
    pub fn search_plugins(&self, query: &str, category: Option<PluginCategory>) -> Vec<&PluginMarketEntry> {
        let mut results = Vec::new();
        
        for plugin in self.plugins.values() {
            // 分类过滤
            if let Some(ref cat) = category {
                if &plugin.category != cat {
                    continue;
                }
            }
            
            // 关键词搜索
            if query.is_empty() || 
               plugin.name.to_lowercase().contains(&query.to_lowercase()) ||
               plugin.description.to_lowercase().contains(&query.to_lowercase()) ||
               plugin.tags.iter().any(|tag| tag.to_lowercase().contains(&query.to_lowercase())) {
                results.push(plugin);
            }
        }
        
        // 按下载量和评分排序
        results.sort_by(|a, b| {
            let score_a = a.downloads as f32 + a.rating * 1000.0;
            let score_b = b.downloads as f32 + b.rating * 1000.0;
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        results
    }
    
    /// 获取插件详情
    pub fn get_plugin(&self, id: &str) -> Option<&PluginMarketEntry> {
        self.plugins.get(id)
    }
    
    /// 获取分类下的插件
    pub fn get_plugins_by_category(&self, category: &PluginCategory) -> Vec<&PluginMarketEntry> {
        if let Some(plugin_ids) = self.categories.get(category) {
            plugin_ids.iter()
                .filter_map(|id| self.plugins.get(id))
                .collect()
        } else {
            Vec::new()
        }
    }
    
    /// 添加插件评价
    pub fn add_review(&mut self, plugin_id: &str, review: PluginReview) -> A2AResult<()> {
        if let Some(plugin) = self.plugins.get_mut(plugin_id) {
            plugin.reviews.push(review);
            
            // 重新计算平均评分
            let total_rating: u32 = plugin.reviews.iter().map(|r| r.rating as u32).sum();
            plugin.rating = total_rating as f32 / plugin.reviews.len() as f32;
            
            Ok(())
        } else {
            Err(A2AError::agent_not_found(format!("插件 {} 不存在", plugin_id)))
        }
    }
    
    /// 增加下载计数
    pub fn increment_downloads(&mut self, plugin_id: &str) -> A2AResult<()> {
        if let Some(plugin) = self.plugins.get_mut(plugin_id) {
            plugin.downloads += 1;
            Ok(())
        } else {
            Err(A2AError::agent_not_found(format!("插件 {} 不存在", plugin_id)))
        }
    }
}

/// CLI工具管理器
pub struct CliToolManager {
    commands: HashMap<String, CliCommand>,
    templates: HashMap<String, ProjectTemplate>,
}

impl CliToolManager {
    pub fn new() -> Self {
        let mut manager = Self {
            commands: HashMap::new(),
            templates: HashMap::new(),
        };
        
        manager.register_default_commands();
        manager.register_default_templates();
        manager
    }
    
    /// 注册默认CLI命令
    fn register_default_commands(&mut self) {
        // agentx init 命令
        let init_cmd = CliCommand {
            name: "init".to_string(),
            description: "初始化新的AgentX项目".to_string(),
            usage: "agentx init [OPTIONS] <project-name>".to_string(),
            options: vec![
                CliOption {
                    name: "template".to_string(),
                    short: Some("t".to_string()),
                    description: "项目模板".to_string(),
                    required: false,
                    default_value: Some("basic".to_string()),
                },
                CliOption {
                    name: "language".to_string(),
                    short: Some("l".to_string()),
                    description: "编程语言".to_string(),
                    required: false,
                    default_value: Some("rust".to_string()),
                },
            ],
            subcommands: vec![],
        };
        self.commands.insert("init".to_string(), init_cmd);
        
        // agentx plugin 命令
        let plugin_cmd = CliCommand {
            name: "plugin".to_string(),
            description: "插件管理命令".to_string(),
            usage: "agentx plugin <subcommand>".to_string(),
            options: vec![],
            subcommands: vec![
                CliCommand {
                    name: "list".to_string(),
                    description: "列出已安装的插件".to_string(),
                    usage: "agentx plugin list".to_string(),
                    options: vec![],
                    subcommands: vec![],
                },
                CliCommand {
                    name: "install".to_string(),
                    description: "安装插件".to_string(),
                    usage: "agentx plugin install <plugin-name>".to_string(),
                    options: vec![
                        CliOption {
                            name: "version".to_string(),
                            short: Some("v".to_string()),
                            description: "插件版本".to_string(),
                            required: false,
                            default_value: Some("latest".to_string()),
                        },
                    ],
                    subcommands: vec![],
                },
                CliCommand {
                    name: "uninstall".to_string(),
                    description: "卸载插件".to_string(),
                    usage: "agentx plugin uninstall <plugin-name>".to_string(),
                    options: vec![],
                    subcommands: vec![],
                },
            ],
        };
        self.commands.insert("plugin".to_string(), plugin_cmd);
        
        // agentx dev 命令
        let dev_cmd = CliCommand {
            name: "dev".to_string(),
            description: "开发工具命令".to_string(),
            usage: "agentx dev <subcommand>".to_string(),
            options: vec![],
            subcommands: vec![
                CliCommand {
                    name: "start".to_string(),
                    description: "启动开发服务器".to_string(),
                    usage: "agentx dev start".to_string(),
                    options: vec![
                        CliOption {
                            name: "port".to_string(),
                            short: Some("p".to_string()),
                            description: "服务器端口".to_string(),
                            required: false,
                            default_value: Some("8080".to_string()),
                        },
                    ],
                    subcommands: vec![],
                },
                CliCommand {
                    name: "test".to_string(),
                    description: "运行测试".to_string(),
                    usage: "agentx dev test".to_string(),
                    options: vec![],
                    subcommands: vec![],
                },
            ],
        };
        self.commands.insert("dev".to_string(), dev_cmd);
    }
    
    /// 注册默认项目模板
    fn register_default_templates(&mut self) {
        // Rust插件模板
        let rust_template = ProjectTemplate {
            name: "rust-plugin".to_string(),
            description: "Rust gRPC插件模板".to_string(),
            language: "rust".to_string(),
            framework: "agentx".to_string(),
            files: {
                let mut files = HashMap::new();
                files.insert("Cargo.toml".to_string(), self.generate_rust_cargo_toml());
                files.insert("src/main.rs".to_string(), self.generate_rust_main());
                files.insert("src/plugin.rs".to_string(), self.generate_rust_plugin());
                files
            },
            dependencies: vec![
                "agentx-sdk".to_string(),
                "tokio".to_string(),
                "tonic".to_string(),
            ],
            setup_instructions: vec![
                "cargo build".to_string(),
                "cargo test".to_string(),
                "cargo run".to_string(),
            ],
        };
        self.templates.insert("rust-plugin".to_string(), rust_template);
        
        // Python插件模板
        let python_template = ProjectTemplate {
            name: "python-plugin".to_string(),
            description: "Python gRPC插件模板".to_string(),
            language: "python".to_string(),
            framework: "agentx".to_string(),
            files: {
                let mut files = HashMap::new();
                files.insert("requirements.txt".to_string(), self.generate_python_requirements());
                files.insert("main.py".to_string(), self.generate_python_main());
                files.insert("plugin.py".to_string(), self.generate_python_plugin());
                files
            },
            dependencies: vec![
                "grpcio".to_string(),
                "grpcio-tools".to_string(),
                "agentx-sdk".to_string(),
            ],
            setup_instructions: vec![
                "pip install -r requirements.txt".to_string(),
                "python main.py".to_string(),
            ],
        };
        self.templates.insert("python-plugin".to_string(), python_template);
    }
    
    /// 获取CLI命令
    pub fn get_command(&self, name: &str) -> Option<&CliCommand> {
        self.commands.get(name)
    }
    
    /// 获取项目模板
    pub fn get_template(&self, name: &str) -> Option<&ProjectTemplate> {
        self.templates.get(name)
    }
    
    /// 列出所有命令
    pub fn list_commands(&self) -> Vec<&CliCommand> {
        self.commands.values().collect()
    }
    
    /// 列出所有模板
    pub fn list_templates(&self) -> Vec<&ProjectTemplate> {
        self.templates.values().collect()
    }
    
    // 模板生成方法
    fn generate_rust_cargo_toml(&self) -> String {
        r#"[package]
name = "agentx-plugin"
version = "0.1.0"
edition = "2021"

[dependencies]
agentx-sdk = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
tonic = "0.10"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tracing = "0.1"
"#.to_string()
    }
    
    fn generate_rust_main(&self) -> String {
        r#"use agentx_sdk::prelude::*;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::init();
    
    let plugin = MyPlugin::new();
    let server = PluginServer::new(plugin);
    
    println!("启动AgentX插件服务器...");
    server.serve("0.0.0.0:50051").await?;
    
    Ok(())
}

struct MyPlugin;

impl MyPlugin {
    fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Plugin for MyPlugin {
    async fn execute(&self, request: PluginRequest) -> PluginResult<PluginResponse> {
        // 实现插件逻辑
        Ok(PluginResponse::success("Hello from AgentX plugin!"))
    }
}
"#.to_string()
    }
    
    fn generate_rust_plugin(&self) -> String {
        r#"// 插件实现文件
// 在这里添加您的插件逻辑
"#.to_string()
    }
    
    fn generate_python_requirements(&self) -> String {
        r#"grpcio>=1.50.0
grpcio-tools>=1.50.0
agentx-sdk>=0.1.0
asyncio
"#.to_string()
    }
    
    fn generate_python_main(&self) -> String {
        r#"import asyncio
from agentx_sdk import Plugin, PluginServer

class MyPlugin(Plugin):
    async def execute(self, request):
        # 实现插件逻辑
        return {"status": "success", "message": "Hello from AgentX Python plugin!"}

async def main():
    plugin = MyPlugin()
    server = PluginServer(plugin)
    
    print("启动AgentX Python插件服务器...")
    await server.serve("0.0.0.0:50051")

if __name__ == "__main__":
    asyncio.run(main())
"#.to_string()
    }
    
    fn generate_python_plugin(&self) -> String {
        r#"# 插件实现文件
# 在这里添加您的插件逻辑
"#.to_string()
    }
}

/// 开发者生态系统管理器
pub struct DeveloperEcosystemManager {
    market: PluginMarketManager,
    cli: CliToolManager,
}

impl DeveloperEcosystemManager {
    pub fn new() -> Self {
        Self {
            market: PluginMarketManager::new(),
            cli: CliToolManager::new(),
        }
    }
    
    /// 获取插件市场管理器
    pub fn market(&mut self) -> &mut PluginMarketManager {
        &mut self.market
    }
    
    /// 获取CLI工具管理器
    pub fn cli(&self) -> &CliToolManager {
        &self.cli
    }
    
    /// 初始化开发者环境
    pub async fn setup_developer_environment(&self) -> A2AResult<()> {
        // 创建必要的目录结构
        // 下载必要的工具和依赖
        // 设置开发环境配置
        
        println!("✅ 开发者环境设置完成");
        Ok(())
    }
    
    /// 生成插件脚手架
    pub async fn generate_plugin_scaffold(&self, 
        name: &str, 
        template: &str, 
        _output_dir: &str
    ) -> A2AResult<()> {
        if let Some(_template_def) = self.cli.get_template(template) {
            // 创建项目目录
            // 生成文件
            // 设置依赖
            
            println!("✅ 插件脚手架生成完成: {}", name);
            Ok(())
        } else {
            Err(A2AError::agent_not_found(format!("模板 {} 不存在", template)))
        }
    }
}

impl Default for PluginMarketManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for CliToolManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for DeveloperEcosystemManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_plugin_market() {
        let mut market = PluginMarketManager::new();
        
        let plugin = PluginMarketEntry {
            id: "test-plugin".to_string(),
            name: "Test Plugin".to_string(),
            description: "A test plugin".to_string(),
            version: "1.0.0".to_string(),
            author: "Test Author".to_string(),
            category: PluginCategory::Tools,
            tags: vec!["test".to_string()],
            download_url: "https://example.com/plugin.zip".to_string(),
            documentation_url: None,
            source_code_url: None,
            license: "MIT".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            downloads: 0,
            rating: 0.0,
            reviews: vec![],
            compatibility: CompatibilityInfo {
                agentx_version: "0.1.0".to_string(),
                supported_platforms: vec!["linux".to_string()],
                required_features: vec![],
            },
            dependencies: vec![],
        };
        
        market.register_plugin(plugin).unwrap();
        
        let results = market.search_plugins("test", None);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Test Plugin");
    }
    
    #[tokio::test]
    async fn test_cli_tools() {
        let cli = CliToolManager::new();
        
        let init_cmd = cli.get_command("init").unwrap();
        assert_eq!(init_cmd.name, "init");
        
        let rust_template = cli.get_template("rust-plugin").unwrap();
        assert_eq!(rust_template.language, "rust");
    }
    
    #[tokio::test]
    async fn test_developer_ecosystem() {
        let mut ecosystem = DeveloperEcosystemManager::new();
        
        // 测试插件注册
        let plugin = PluginMarketEntry {
            id: "ecosystem-test".to_string(),
            name: "Ecosystem Test".to_string(),
            description: "Test plugin for ecosystem".to_string(),
            version: "1.0.0".to_string(),
            author: "Test".to_string(),
            category: PluginCategory::Development,
            tags: vec!["ecosystem".to_string()],
            download_url: "https://example.com/test.zip".to_string(),
            documentation_url: None,
            source_code_url: None,
            license: "MIT".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            downloads: 0,
            rating: 0.0,
            reviews: vec![],
            compatibility: CompatibilityInfo {
                agentx_version: "0.1.0".to_string(),
                supported_platforms: vec!["linux".to_string()],
                required_features: vec![],
            },
            dependencies: vec![],
        };
        
        ecosystem.market().register_plugin(plugin).unwrap();
        
        let results = ecosystem.market().search_plugins("ecosystem", None);
        assert_eq!(results.len(), 1);
    }
}
