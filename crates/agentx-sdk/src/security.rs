//! 插件安全隔离和权限控制
//! 
//! 提供插件运行时的安全隔离、权限控制和资源限制功能

use agentx_a2a::{A2AResult, A2AError, TrustLevel};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::SystemTime;
use tokio::sync::RwLock;
use std::sync::Arc;
use tracing::{debug, warn, error};

/// 插件安全管理器
pub struct PluginSecurityManager {
    /// 权限策略
    permission_policies: Arc<RwLock<HashMap<String, PermissionPolicy>>>,
    /// 资源限制
    resource_limits: Arc<RwLock<HashMap<String, ResourceLimits>>>,
    /// 访问控制列表
    access_control: Arc<RwLock<HashMap<String, AccessControlList>>>,
    /// 安全审计日志
    audit_log: Arc<RwLock<Vec<SecurityAuditEntry>>>,
    /// 配置
    config: SecurityConfig,
}

/// 权限策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionPolicy {
    /// 插件ID
    pub plugin_id: String,
    /// 允许的操作
    pub allowed_operations: HashSet<Operation>,
    /// 禁止的操作
    pub denied_operations: HashSet<Operation>,
    /// 可访问的资源
    pub accessible_resources: HashSet<Resource>,
    /// 信任级别
    pub trust_level: TrustLevel,
    /// 策略创建时间
    pub created_at: SystemTime,
    /// 策略过期时间
    pub expires_at: Option<SystemTime>,
}

/// 操作类型
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum Operation {
    /// 读取消息
    ReadMessage,
    /// 发送消息
    SendMessage,
    /// 创建Agent
    CreateAgent,
    /// 删除Agent
    DeleteAgent,
    /// 修改Agent配置
    ModifyAgentConfig,
    /// 访问文件系统
    AccessFileSystem,
    /// 网络访问
    NetworkAccess,
    /// 执行系统命令
    ExecuteCommand,
    /// 访问环境变量
    AccessEnvironment,
    /// 访问数据库
    AccessDatabase,
    /// 调用外部API
    CallExternalAPI,
    /// 自定义操作
    Custom(String),
}

/// 资源类型
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum Resource {
    /// Agent资源
    Agent(String),
    /// 消息资源
    Message(String),
    /// 文件资源
    File(String),
    /// 网络资源
    Network(String),
    /// 数据库资源
    Database(String),
    /// API资源
    API(String),
    /// 自定义资源
    Custom(String, String),
}

/// 资源限制
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// 插件ID
    pub plugin_id: String,
    /// 最大内存使用量（字节）
    pub max_memory_bytes: Option<u64>,
    /// 最大CPU使用率（0.0-1.0）
    pub max_cpu_usage: Option<f64>,
    /// 最大网络带宽（字节/秒）
    pub max_network_bandwidth: Option<u64>,
    /// 最大文件描述符数量
    pub max_file_descriptors: Option<u32>,
    /// 最大并发连接数
    pub max_concurrent_connections: Option<u32>,
    /// 请求速率限制（请求/秒）
    pub rate_limit_per_second: Option<u32>,
    /// 最大运行时间（秒）
    pub max_runtime_seconds: Option<u64>,
}

/// 访问控制列表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessControlList {
    /// 插件ID
    pub plugin_id: String,
    /// 允许访问的插件列表
    pub allowed_plugins: HashSet<String>,
    /// 禁止访问的插件列表
    pub denied_plugins: HashSet<String>,
    /// 允许访问的Agent列表
    pub allowed_agents: HashSet<String>,
    /// 禁止访问的Agent列表
    pub denied_agents: HashSet<String>,
    /// 允许的IP地址范围
    pub allowed_ip_ranges: Vec<String>,
    /// 禁止的IP地址范围
    pub denied_ip_ranges: Vec<String>,
}

/// 安全审计条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAuditEntry {
    /// 时间戳
    pub timestamp: SystemTime,
    /// 插件ID
    pub plugin_id: String,
    /// 操作类型
    pub operation: Operation,
    /// 资源
    pub resource: Option<Resource>,
    /// 结果
    pub result: SecurityResult,
    /// 详细信息
    pub details: String,
    /// 客户端IP
    pub client_ip: Option<String>,
}

/// 安全检查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityResult {
    /// 允许
    Allowed,
    /// 拒绝
    Denied(String),
    /// 错误
    Error(String),
}

/// 安全配置
#[derive(Debug, Clone)]
pub struct SecurityConfig {
    /// 是否启用权限检查
    pub enable_permission_check: bool,
    /// 是否启用资源限制
    pub enable_resource_limits: bool,
    /// 是否启用访问控制
    pub enable_access_control: bool,
    /// 是否启用安全审计
    pub enable_security_audit: bool,
    /// 审计日志最大条目数
    pub max_audit_entries: usize,
    /// 默认信任级别
    pub default_trust_level: TrustLevel,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enable_permission_check: true,
            enable_resource_limits: true,
            enable_access_control: true,
            enable_security_audit: true,
            max_audit_entries: 10000,
            default_trust_level: TrustLevel::Public,
        }
    }
}

impl PluginSecurityManager {
    /// 创建新的安全管理器
    pub fn new(config: SecurityConfig) -> Self {
        Self {
            permission_policies: Arc::new(RwLock::new(HashMap::new())),
            resource_limits: Arc::new(RwLock::new(HashMap::new())),
            access_control: Arc::new(RwLock::new(HashMap::new())),
            audit_log: Arc::new(RwLock::new(Vec::new())),
            config,
        }
    }

    /// 设置插件权限策略
    pub async fn set_permission_policy(&self, policy: PermissionPolicy) -> A2AResult<()> {
        debug!("设置插件 {} 的权限策略", policy.plugin_id);
        
        self.permission_policies.write().await.insert(policy.plugin_id.clone(), policy);
        Ok(())
    }

    /// 设置插件资源限制
    pub async fn set_resource_limits(&self, limits: ResourceLimits) -> A2AResult<()> {
        debug!("设置插件 {} 的资源限制", limits.plugin_id);
        
        self.resource_limits.write().await.insert(limits.plugin_id.clone(), limits);
        Ok(())
    }

    /// 设置插件访问控制列表
    pub async fn set_access_control(&self, acl: AccessControlList) -> A2AResult<()> {
        debug!("设置插件 {} 的访问控制列表", acl.plugin_id);
        
        self.access_control.write().await.insert(acl.plugin_id.clone(), acl);
        Ok(())
    }

    /// 检查操作权限
    pub async fn check_permission(
        &self,
        plugin_id: &str,
        operation: &Operation,
        resource: Option<&Resource>,
    ) -> A2AResult<bool> {
        if !self.config.enable_permission_check {
            return Ok(true);
        }

        let policy = {
            let policies = self.permission_policies.read().await;
            policies.get(plugin_id).cloned()
        };

        let result = match policy {
            Some(policy) => {
                // 检查策略是否过期
                if let Some(expires_at) = policy.expires_at {
                    if SystemTime::now() > expires_at {
                        SecurityResult::Denied("权限策略已过期".to_string())
                    } else {
                        self.evaluate_permission(&policy, operation, resource).await
                    }
                } else {
                    self.evaluate_permission(&policy, operation, resource).await
                }
            }
            None => {
                // 没有策略，使用默认行为
                match self.config.default_trust_level {
                    TrustLevel::Public => SecurityResult::Allowed,
                    _ => SecurityResult::Denied("没有权限策略".to_string()),
                }
            }
        };

        // 记录审计日志
        if self.config.enable_security_audit {
            self.log_security_event(plugin_id, operation.clone(), resource.cloned(), result.clone()).await;
        }

        match result {
            SecurityResult::Allowed => Ok(true),
            SecurityResult::Denied(reason) => {
                warn!("插件 {} 操作被拒绝: {}", plugin_id, reason);
                Ok(false)
            }
            SecurityResult::Error(error) => {
                error!("插件 {} 权限检查错误: {}", plugin_id, error);
                Err(A2AError::internal(error))
            }
        }
    }

    /// 检查访问控制
    pub async fn check_access_control(
        &self,
        plugin_id: &str,
        target_plugin: Option<&str>,
        target_agent: Option<&str>,
        client_ip: Option<&str>,
    ) -> A2AResult<bool> {
        if !self.config.enable_access_control {
            return Ok(true);
        }

        let acl = {
            let access_controls = self.access_control.read().await;
            access_controls.get(plugin_id).cloned()
        };

        let acl = match acl {
            Some(acl) => acl,
            None => return Ok(true), // 没有ACL，默认允许
        };

        // 检查插件访问权限
        if let Some(target) = target_plugin {
            if acl.denied_plugins.contains(target) {
                return Ok(false);
            }
            if !acl.allowed_plugins.is_empty() && !acl.allowed_plugins.contains(target) {
                return Ok(false);
            }
        }

        // 检查Agent访问权限
        if let Some(target) = target_agent {
            if acl.denied_agents.contains(target) {
                return Ok(false);
            }
            if !acl.allowed_agents.is_empty() && !acl.allowed_agents.contains(target) {
                return Ok(false);
            }
        }

        // 检查IP访问权限
        if let Some(ip) = client_ip {
            // 简化的IP检查，实际实现应该支持CIDR等
            if acl.denied_ip_ranges.iter().any(|range| ip.starts_with(range)) {
                return Ok(false);
            }
            if !acl.allowed_ip_ranges.is_empty() && 
               !acl.allowed_ip_ranges.iter().any(|range| ip.starts_with(range)) {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// 获取插件资源限制
    pub async fn get_resource_limits(&self, plugin_id: &str) -> Option<ResourceLimits> {
        self.resource_limits.read().await.get(plugin_id).cloned()
    }

    /// 获取安全审计日志
    pub async fn get_audit_log(&self, plugin_id: Option<&str>, limit: Option<usize>) -> Vec<SecurityAuditEntry> {
        let log = self.audit_log.read().await;
        let filtered: Vec<_> = match plugin_id {
            Some(id) => log.iter().filter(|entry| entry.plugin_id == id).cloned().collect(),
            None => log.clone(),
        };

        match limit {
            Some(n) => filtered.into_iter().rev().take(n).collect(),
            None => filtered,
        }
    }

    // 私有方法

    async fn evaluate_permission(
        &self,
        policy: &PermissionPolicy,
        operation: &Operation,
        resource: Option<&Resource>,
    ) -> SecurityResult {
        // 检查是否明确禁止
        if policy.denied_operations.contains(operation) {
            return SecurityResult::Denied("操作被明确禁止".to_string());
        }

        // 检查是否明确允许
        if policy.allowed_operations.contains(operation) {
            // 如果指定了资源，检查资源访问权限
            if let Some(res) = resource {
                if policy.accessible_resources.contains(res) {
                    SecurityResult::Allowed
                } else {
                    SecurityResult::Denied("资源访问被拒绝".to_string())
                }
            } else {
                SecurityResult::Allowed
            }
        } else {
            // 根据信任级别决定默认行为
            match policy.trust_level {
                TrustLevel::Public => SecurityResult::Denied("操作未明确允许".to_string()),
                TrustLevel::Verified => SecurityResult::Denied("操作未明确允许".to_string()),
                TrustLevel::Trusted => SecurityResult::Allowed,
                TrustLevel::Internal => SecurityResult::Allowed,
            }
        }
    }

    async fn log_security_event(
        &self,
        plugin_id: &str,
        operation: Operation,
        resource: Option<Resource>,
        result: SecurityResult,
    ) {
        let entry = SecurityAuditEntry {
            timestamp: SystemTime::now(),
            plugin_id: plugin_id.to_string(),
            operation,
            resource,
            result,
            details: "权限检查".to_string(),
            client_ip: None, // 这里可以从上下文获取
        };

        let mut log = self.audit_log.write().await;
        log.push(entry);

        // 限制日志大小
        if log.len() > self.config.max_audit_entries {
            log.remove(0);
        }
    }
}

/// 创建默认权限策略
pub fn create_default_permission_policy(plugin_id: String, trust_level: TrustLevel) -> PermissionPolicy {
    let mut allowed_operations = HashSet::new();
    let accessible_resources = HashSet::new();

    // 根据信任级别设置默认权限
    match trust_level {
        TrustLevel::Public => {
            allowed_operations.insert(Operation::ReadMessage);
            allowed_operations.insert(Operation::SendMessage);
        }
        TrustLevel::Verified => {
            allowed_operations.insert(Operation::ReadMessage);
            allowed_operations.insert(Operation::SendMessage);
            allowed_operations.insert(Operation::CreateAgent);
        }
        TrustLevel::Trusted => {
            allowed_operations.insert(Operation::ReadMessage);
            allowed_operations.insert(Operation::SendMessage);
            allowed_operations.insert(Operation::CreateAgent);
            allowed_operations.insert(Operation::DeleteAgent);
            allowed_operations.insert(Operation::ModifyAgentConfig);
        }
        TrustLevel::Internal => {
            // 内部插件拥有所有权限
            allowed_operations.insert(Operation::ReadMessage);
            allowed_operations.insert(Operation::SendMessage);
            allowed_operations.insert(Operation::CreateAgent);
            allowed_operations.insert(Operation::DeleteAgent);
            allowed_operations.insert(Operation::ModifyAgentConfig);
            allowed_operations.insert(Operation::AccessFileSystem);
            allowed_operations.insert(Operation::NetworkAccess);
        }
    }

    PermissionPolicy {
        plugin_id,
        allowed_operations,
        denied_operations: HashSet::new(),
        accessible_resources,
        trust_level,
        created_at: SystemTime::now(),
        expires_at: None,
    }
}
