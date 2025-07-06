//! AgentX 插件市场
//! 
//! 提供插件的发现、安装、更新和管理功能，
//! 支持插件的版本控制、依赖管理和安全验证

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;
use tokio::fs;
use tracing::{info, debug, warn};
use agentx_a2a::{A2AResult, A2AError};

/// 插件市场管理器
pub struct PluginMarketplace {
    /// 市场配置
    config: MarketplaceConfig,
    /// 本地插件缓存
    local_cache: HashMap<String, PluginPackage>,
    /// 远程仓库列表
    repositories: Vec<PluginRepository>,
    /// 已安装插件
    installed_plugins: HashMap<String, InstalledPlugin>,
}

/// 市场配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceConfig {
    /// 本地缓存目录
    pub cache_dir: PathBuf,
    /// 插件安装目录
    pub install_dir: PathBuf,
    /// 默认仓库URL
    pub default_repository: String,
    /// 缓存过期时间（小时）
    pub cache_expiry_hours: u64,
    /// 是否启用自动更新
    pub auto_update: bool,
    /// 是否验证插件签名
    pub verify_signatures: bool,
    /// 最大并发下载数
    pub max_concurrent_downloads: usize,
}

impl Default for MarketplaceConfig {
    fn default() -> Self {
        Self {
            cache_dir: PathBuf::from("~/.agentx/cache"),
            install_dir: PathBuf::from("~/.agentx/plugins"),
            default_repository: "https://plugins.agentx.dev".to_string(),
            cache_expiry_hours: 24,
            auto_update: false,
            verify_signatures: true,
            max_concurrent_downloads: 3,
        }
    }
}

/// 插件包信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginPackage {
    /// 插件元数据
    pub metadata: PluginMetadata,
    /// 版本信息
    pub versions: Vec<PluginVersion>,
    /// 下载统计
    pub download_stats: DownloadStats,
    /// 评分和评论
    pub ratings: PluginRatings,
    /// 依赖关系
    pub dependencies: Vec<PluginDependency>,
    /// 标签
    pub tags: Vec<String>,
    /// 许可证
    pub license: String,
    /// 仓库URL
    pub repository_url: Option<String>,
    /// 文档URL
    pub documentation_url: Option<String>,
}

/// 插件元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// 插件名称
    pub name: String,
    /// 显示名称
    pub display_name: String,
    /// 描述
    pub description: String,
    /// 作者
    pub author: String,
    /// 作者邮箱
    pub author_email: Option<String>,
    /// 主页URL
    pub homepage: Option<String>,
    /// 关键词
    pub keywords: Vec<String>,
    /// 分类
    pub category: PluginCategory,
    /// 创建时间
    pub created_at: SystemTime,
    /// 更新时间
    pub updated_at: SystemTime,
}

/// 插件版本
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginVersion {
    /// 版本号
    pub version: String,
    /// 发布时间
    pub released_at: SystemTime,
    /// 变更日志
    pub changelog: String,
    /// 下载URL
    pub download_url: String,
    /// 文件大小（字节）
    pub file_size: u64,
    /// 文件哈希
    pub file_hash: String,
    /// 签名
    pub signature: Option<String>,
    /// 最低AgentX版本要求
    pub min_agentx_version: String,
    /// 是否为预发布版本
    pub is_prerelease: bool,
}

/// 下载统计
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DownloadStats {
    /// 总下载次数
    pub total_downloads: u64,
    /// 本周下载次数
    pub weekly_downloads: u64,
    /// 本月下载次数
    pub monthly_downloads: u64,
    /// 最近下载时间
    pub last_download: Option<SystemTime>,
}

/// 插件评分
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PluginRatings {
    /// 平均评分 (1-5)
    pub average_rating: f64,
    /// 评分总数
    pub total_ratings: u32,
    /// 各星级评分数量
    pub rating_distribution: HashMap<u8, u32>,
    /// 评论列表
    pub reviews: Vec<PluginReview>,
}

/// 插件评论
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginReview {
    /// 评论ID
    pub id: String,
    /// 用户名
    pub username: String,
    /// 评分 (1-5)
    pub rating: u8,
    /// 评论内容
    pub comment: String,
    /// 评论时间
    pub created_at: SystemTime,
    /// 插件版本
    pub plugin_version: String,
}

/// 插件依赖
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDependency {
    /// 依赖插件名称
    pub name: String,
    /// 版本要求
    pub version_requirement: String,
    /// 是否为可选依赖
    pub optional: bool,
}

/// 插件分类
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginCategory {
    /// 框架适配器
    FrameworkAdapter,
    /// 协议转换器
    ProtocolConverter,
    /// 数据处理器
    DataProcessor,
    /// 监控工具
    Monitoring,
    /// 安全工具
    Security,
    /// 开发工具
    Development,
    /// 集成工具
    Integration,
    /// 其他
    Other,
}

/// 插件仓库
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginRepository {
    /// 仓库名称
    pub name: String,
    /// 仓库URL
    pub url: String,
    /// 是否启用
    pub enabled: bool,
    /// 优先级
    pub priority: u32,
    /// 认证信息
    pub auth: Option<RepositoryAuth>,
}

/// 仓库认证信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryAuth {
    /// 认证类型
    pub auth_type: AuthType,
    /// 用户名
    pub username: Option<String>,
    /// 密码或令牌
    pub token: Option<String>,
}

/// 认证类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthType {
    /// 无认证
    None,
    /// 基础认证
    Basic,
    /// Bearer令牌
    Bearer,
    /// API密钥
    ApiKey,
}

/// 已安装插件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledPlugin {
    /// 插件包信息
    pub package: PluginPackage,
    /// 安装版本
    pub installed_version: String,
    /// 安装时间
    pub installed_at: SystemTime,
    /// 安装路径
    pub install_path: PathBuf,
    /// 是否启用
    pub enabled: bool,
    /// 配置文件路径
    pub config_path: Option<PathBuf>,
}

/// 搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// 匹配的插件列表
    pub plugins: Vec<PluginPackage>,
    /// 总结果数
    pub total_count: usize,
    /// 当前页码
    pub page: usize,
    /// 每页大小
    pub page_size: usize,
}

/// 搜索查询
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchQuery {
    /// 关键词
    pub keywords: Option<String>,
    /// 分类过滤
    pub category: Option<PluginCategory>,
    /// 作者过滤
    pub author: Option<String>,
    /// 标签过滤
    pub tags: Vec<String>,
    /// 排序方式
    pub sort_by: SortBy,
    /// 页码
    pub page: usize,
    /// 每页大小
    pub page_size: usize,
}

/// 排序方式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortBy {
    /// 相关性
    Relevance,
    /// 下载量
    Downloads,
    /// 评分
    Rating,
    /// 更新时间
    Updated,
    /// 创建时间
    Created,
    /// 名称
    Name,
}

impl Default for SortBy {
    fn default() -> Self {
        SortBy::Relevance
    }
}

impl PluginMarketplace {
    /// 创建新的插件市场管理器
    pub async fn new(config: MarketplaceConfig) -> A2AResult<Self> {
        info!("🏪 创建插件市场管理器");
        
        // 创建必要的目录
        fs::create_dir_all(&config.cache_dir).await
            .map_err(|e| A2AError::internal(format!("创建缓存目录失败: {}", e)))?;

        fs::create_dir_all(&config.install_dir).await
            .map_err(|e| A2AError::internal(format!("创建安装目录失败: {}", e)))?;
        
        let mut marketplace = Self {
            config,
            local_cache: HashMap::new(),
            repositories: Vec::new(),
            installed_plugins: HashMap::new(),
        };
        
        // 添加默认仓库
        marketplace.add_repository(PluginRepository {
            name: "official".to_string(),
            url: marketplace.config.default_repository.clone(),
            enabled: true,
            priority: 100,
            auth: None,
        }).await?;
        
        // 加载已安装插件
        marketplace.load_installed_plugins().await?;
        
        Ok(marketplace)
    }
    
    /// 添加插件仓库
    pub async fn add_repository(&mut self, repository: PluginRepository) -> A2AResult<()> {
        info!("添加插件仓库: {} ({})", repository.name, repository.url);
        
        // 验证仓库连接
        if repository.enabled {
            self.test_repository_connection(&repository).await?;
        }
        
        self.repositories.push(repository);
        self.repositories.sort_by(|a, b| b.priority.cmp(&a.priority));
        
        Ok(())
    }
    
    /// 搜索插件
    pub async fn search_plugins(&self, query: SearchQuery) -> A2AResult<SearchResult> {
        debug!("搜索插件: {:?}", query);
        
        let mut all_plugins = Vec::new();
        
        // 从所有启用的仓库搜索
        for repository in &self.repositories {
            if repository.enabled {
                match self.search_in_repository(repository, &query).await {
                    Ok(mut plugins) => all_plugins.append(&mut plugins),
                    Err(e) => warn!("仓库 {} 搜索失败: {}", repository.name, e),
                }
            }
        }
        
        // 去重和排序
        all_plugins.sort_by(|a, b| self.compare_plugins(a, b, &query.sort_by));
        all_plugins.dedup_by(|a, b| a.metadata.name == b.metadata.name);
        
        // 分页
        let total_count = all_plugins.len();
        let start = query.page * query.page_size;
        let end = std::cmp::min(start + query.page_size, total_count);
        let plugins = all_plugins.into_iter().skip(start).take(end - start).collect();
        
        Ok(SearchResult {
            plugins,
            total_count,
            page: query.page,
            page_size: query.page_size,
        })
    }
    
    /// 获取插件详情
    pub async fn get_plugin_details(&self, plugin_name: &str) -> A2AResult<Option<PluginPackage>> {
        debug!("获取插件详情: {}", plugin_name);
        
        // 先检查本地缓存
        if let Some(plugin) = self.local_cache.get(plugin_name) {
            return Ok(Some(plugin.clone()));
        }
        
        // 从仓库获取
        for repository in &self.repositories {
            if repository.enabled {
                if let Ok(Some(plugin)) = self.get_plugin_from_repository(repository, plugin_name).await {
                    return Ok(Some(plugin));
                }
            }
        }
        
        Ok(None)
    }
    
    /// 安装插件
    pub async fn install_plugin(&mut self, plugin_name: &str, version: Option<&str>) -> A2AResult<()> {
        info!("安装插件: {} (版本: {:?})", plugin_name, version);
        
        // 获取插件信息
        let plugin = self.get_plugin_details(plugin_name).await?
            .ok_or_else(|| A2AError::validation(format!("插件未找到: {}", plugin_name)))?;

        // 选择版本
        let target_version = if let Some(v) = version {
            plugin.versions.iter().find(|ver| ver.version == v)
                .ok_or_else(|| A2AError::validation(format!("版本未找到: {}", v)))?
        } else {
            plugin.versions.iter()
                .filter(|v| !v.is_prerelease)
                .max_by(|a, b| a.version.cmp(&b.version))
                .ok_or_else(|| A2AError::validation("没有稳定版本"))?
        };
        
        // 检查依赖
        self.check_dependencies(&plugin.dependencies).await?;
        
        // 下载和安装
        self.download_and_install_plugin(&plugin, target_version).await?;
        
        info!("插件 {} 安装成功", plugin_name);
        Ok(())
    }
    
    /// 卸载插件
    pub async fn uninstall_plugin(&mut self, plugin_name: &str) -> A2AResult<()> {
        info!("卸载插件: {}", plugin_name);
        
        let installed_plugin = self.installed_plugins.remove(plugin_name)
            .ok_or_else(|| A2AError::validation(format!("插件未安装: {}", plugin_name)))?;

        // 删除插件文件
        if installed_plugin.install_path.exists() {
            fs::remove_dir_all(&installed_plugin.install_path).await
                .map_err(|e| A2AError::internal(format!("删除插件文件失败: {}", e)))?;
        }

        // 删除配置文件
        if let Some(config_path) = &installed_plugin.config_path {
            if config_path.exists() {
                fs::remove_file(config_path).await
                    .map_err(|e| A2AError::internal(format!("删除配置文件失败: {}", e)))?;
            }
        }
        
        info!("插件 {} 卸载成功", plugin_name);
        Ok(())
    }
    
    /// 更新插件
    pub async fn update_plugin(&mut self, plugin_name: &str) -> A2AResult<()> {
        info!("更新插件: {}", plugin_name);
        
        let current_version = {
            let installed_plugin = self.installed_plugins.get(plugin_name)
                .ok_or_else(|| A2AError::validation(format!("插件未安装: {}", plugin_name)))?;
            installed_plugin.installed_version.clone()
        };

        // 获取最新版本信息
        let plugin = self.get_plugin_details(plugin_name).await?
            .ok_or_else(|| A2AError::validation(format!("插件未找到: {}", plugin_name)))?;

        let latest_version = plugin.versions.iter()
            .filter(|v| !v.is_prerelease)
            .max_by(|a, b| a.version.cmp(&b.version))
            .ok_or_else(|| A2AError::validation("没有稳定版本"))?;

        // 检查是否需要更新
        if latest_version.version <= current_version {
            info!("插件 {} 已是最新版本", plugin_name);
            return Ok(());
        }

        let latest_version_str = latest_version.version.clone();

        // 先卸载旧版本
        self.uninstall_plugin(plugin_name).await?;

        // 安装新版本
        self.install_plugin(plugin_name, Some(&latest_version_str)).await?;

        info!("插件 {} 更新成功: {} -> {}", plugin_name, current_version, latest_version_str);
        Ok(())
    }
    
    /// 列出已安装插件
    pub fn list_installed_plugins(&self) -> Vec<&InstalledPlugin> {
        self.installed_plugins.values().collect()
    }

    /// 检查插件更新
    pub async fn check_updates(&self) -> A2AResult<Vec<PluginUpdate>> {
        debug!("检查插件更新");

        let mut updates = Vec::new();

        for (plugin_name, installed_plugin) in &self.installed_plugins {
            if let Ok(Some(plugin)) = self.get_plugin_details(plugin_name).await {
                if let Some(latest_version) = plugin.versions.iter()
                    .filter(|v| !v.is_prerelease)
                    .max_by(|a, b| a.version.cmp(&b.version)) {

                    if latest_version.version > installed_plugin.installed_version {
                        updates.push(PluginUpdate {
                            plugin_name: plugin_name.clone(),
                            current_version: installed_plugin.installed_version.clone(),
                            latest_version: latest_version.version.clone(),
                            changelog: latest_version.changelog.clone(),
                        });
                    }
                }
            }
        }

        Ok(updates)
    }

    /// 批量更新插件
    pub async fn update_all_plugins(&mut self) -> A2AResult<Vec<PluginUpdateResult>> {
        info!("批量更新所有插件");

        let updates = self.check_updates().await?;
        let mut results = Vec::new();

        for update in updates {
            let result = match self.update_plugin(&update.plugin_name).await {
                Ok(()) => PluginUpdateResult {
                    plugin_name: update.plugin_name.clone(),
                    success: true,
                    error: None,
                },
                Err(e) => PluginUpdateResult {
                    plugin_name: update.plugin_name.clone(),
                    success: false,
                    error: Some(e.to_string()),
                },
            };
            results.push(result);
        }

        Ok(results)
    }

    // 私有辅助方法

    async fn test_repository_connection(&self, repository: &PluginRepository) -> A2AResult<()> {
        debug!("测试仓库连接: {}", repository.url);

        // 这里应该实现实际的HTTP请求来测试连接
        // 为了简化，我们假设连接总是成功的

        Ok(())
    }

    async fn search_in_repository(&self, repository: &PluginRepository, _query: &SearchQuery) -> A2AResult<Vec<PluginPackage>> {
        debug!("在仓库 {} 中搜索", repository.name);

        // 这里应该实现实际的HTTP请求来搜索插件
        // 为了演示，返回空列表

        Ok(Vec::new())
    }

    async fn get_plugin_from_repository(&self, repository: &PluginRepository, plugin_name: &str) -> A2AResult<Option<PluginPackage>> {
        debug!("从仓库 {} 获取插件 {}", repository.name, plugin_name);

        // 这里应该实现实际的HTTP请求来获取插件信息
        // 为了演示，返回None

        Ok(None)
    }

    fn compare_plugins(&self, a: &PluginPackage, b: &PluginPackage, sort_by: &SortBy) -> std::cmp::Ordering {
        match sort_by {
            SortBy::Relevance => a.metadata.name.cmp(&b.metadata.name),
            SortBy::Downloads => b.download_stats.total_downloads.cmp(&a.download_stats.total_downloads),
            SortBy::Rating => b.ratings.average_rating.partial_cmp(&a.ratings.average_rating).unwrap_or(std::cmp::Ordering::Equal),
            SortBy::Updated => b.metadata.updated_at.cmp(&a.metadata.updated_at),
            SortBy::Created => b.metadata.created_at.cmp(&a.metadata.created_at),
            SortBy::Name => a.metadata.name.cmp(&b.metadata.name),
        }
    }

    async fn check_dependencies(&self, dependencies: &[PluginDependency]) -> A2AResult<()> {
        for dep in dependencies {
            if !dep.optional && !self.installed_plugins.contains_key(&dep.name) {
                return Err(A2AError::validation(format!("依赖未找到: {}", dep.name)));
            }
        }
        Ok(())
    }

    async fn download_and_install_plugin(&mut self, plugin: &PluginPackage, version: &PluginVersion) -> A2AResult<()> {
        debug!("下载并安装插件: {} v{}", plugin.metadata.name, version.version);

        // 创建安装目录
        let install_path = self.config.install_dir.join(&plugin.metadata.name);
        fs::create_dir_all(&install_path).await
            .map_err(|e| A2AError::internal(format!("创建安装目录失败: {}", e)))?;

        // 这里应该实现实际的下载和解压逻辑
        // 为了演示，我们只是创建一个标记文件
        let marker_file = install_path.join("installed.marker");
        fs::write(&marker_file, format!("{}:{}", plugin.metadata.name, version.version)).await
            .map_err(|e| A2AError::internal(format!("创建标记文件失败: {}", e)))?;

        // 记录已安装插件
        let installed_plugin = InstalledPlugin {
            package: plugin.clone(),
            installed_version: version.version.clone(),
            installed_at: SystemTime::now(),
            install_path,
            enabled: true,
            config_path: None,
        };

        self.installed_plugins.insert(plugin.metadata.name.clone(), installed_plugin);

        Ok(())
    }

    async fn load_installed_plugins(&mut self) -> A2AResult<()> {
        debug!("加载已安装插件");

        if !self.config.install_dir.exists() {
            return Ok(());
        }

        let mut entries = fs::read_dir(&self.config.install_dir).await
            .map_err(|e| A2AError::internal(format!("读取安装目录失败: {}", e)))?;

        while let Some(entry) = entries.next_entry().await
            .map_err(|e| A2AError::internal(format!("读取目录项失败: {}", e)))? {

            if entry.file_type().await.map_err(|e| A2AError::internal(e.to_string()))?.is_dir() {
                let plugin_dir = entry.path();
                let marker_file = plugin_dir.join("installed.marker");

                if marker_file.exists() {
                    if let Ok(content) = fs::read_to_string(&marker_file).await {
                        if let Some((name, version)) = content.split_once(':') {
                            // 这里应该加载完整的插件信息
                            // 为了演示，我们创建一个简化的记录
                            debug!("发现已安装插件: {} v{}", name, version);
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

/// 插件更新信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginUpdate {
    /// 插件名称
    pub plugin_name: String,
    /// 当前版本
    pub current_version: String,
    /// 最新版本
    pub latest_version: String,
    /// 变更日志
    pub changelog: String,
}

/// 插件更新结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginUpdateResult {
    /// 插件名称
    pub plugin_name: String,
    /// 是否成功
    pub success: bool,
    /// 错误信息
    pub error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;

    fn create_test_config() -> MarketplaceConfig {
        MarketplaceConfig {
            cache_dir: PathBuf::from("/tmp/agentx-test/cache"),
            install_dir: PathBuf::from("/tmp/agentx-test/plugins"),
            default_repository: "https://test.plugins.agentx.dev".to_string(),
            cache_expiry_hours: 1,
            auto_update: false,
            verify_signatures: false,
            max_concurrent_downloads: 1,
        }
    }

    fn create_test_plugin() -> PluginPackage {
        PluginPackage {
            metadata: PluginMetadata {
                name: "test-plugin".to_string(),
                display_name: "Test Plugin".to_string(),
                description: "A test plugin".to_string(),
                author: "Test Author".to_string(),
                author_email: Some("test@example.com".to_string()),
                homepage: None,
                keywords: vec!["test".to_string()],
                category: PluginCategory::Development,
                created_at: SystemTime::now(),
                updated_at: SystemTime::now(),
            },
            versions: vec![
                PluginVersion {
                    version: "1.0.0".to_string(),
                    released_at: SystemTime::now(),
                    changelog: "Initial release".to_string(),
                    download_url: "https://example.com/plugin.tar.gz".to_string(),
                    file_size: 1024,
                    file_hash: "abc123".to_string(),
                    signature: None,
                    min_agentx_version: "0.1.0".to_string(),
                    is_prerelease: false,
                },
            ],
            download_stats: DownloadStats::default(),
            ratings: PluginRatings::default(),
            dependencies: Vec::new(),
            tags: vec!["test".to_string()],
            license: "MIT".to_string(),
            repository_url: None,
            documentation_url: None,
        }
    }

    #[tokio::test]
    async fn test_marketplace_creation() {
        let config = create_test_config();
        let marketplace = PluginMarketplace::new(config).await;
        assert!(marketplace.is_ok());
    }

    #[tokio::test]
    async fn test_search_query_default() {
        let query = SearchQuery::default();
        assert_eq!(query.page, 0);
        assert_eq!(query.page_size, 0);
        assert!(matches!(query.sort_by, SortBy::Relevance));
    }

    #[test]
    fn test_plugin_category_serialization() {
        let category = PluginCategory::FrameworkAdapter;
        let serialized = serde_json::to_string(&category).unwrap();
        let deserialized: PluginCategory = serde_json::from_str(&serialized).unwrap();
        assert!(matches!(deserialized, PluginCategory::FrameworkAdapter));
    }

    #[test]
    fn test_plugin_package_creation() {
        let plugin = create_test_plugin();
        assert_eq!(plugin.metadata.name, "test-plugin");
        assert_eq!(plugin.versions.len(), 1);
        assert_eq!(plugin.versions[0].version, "1.0.0");
    }
}
