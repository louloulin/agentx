//! 健康检查器
//! 
//! 监控Agent和节点的健康状态

use crate::config::HealthCheckConfig;
use crate::error::{ClusterError, ClusterResult};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, debug, warn, error};

/// 健康状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// 健康
    Healthy,
    /// 不健康
    Unhealthy,
    /// 未知
    Unknown,
    /// 检查中
    Checking,
}

/// 健康检查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    /// 目标ID
    pub target_id: String,
    /// 健康状态
    pub status: HealthStatus,
    /// 响应时间（毫秒）
    pub response_time: Option<u64>,
    /// 错误信息
    pub error: Option<String>,
    /// 检查时间
    pub checked_at: chrono::DateTime<chrono::Utc>,
    /// 额外信息
    pub details: std::collections::HashMap<String, String>,
}

/// 健康检查目标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckTarget {
    /// 目标ID
    pub id: String,
    /// 目标端点
    pub endpoint: String,
    /// 检查间隔
    pub interval: std::time::Duration,
    /// 超时时间
    pub timeout: std::time::Duration,
    /// 重试次数
    pub retries: u32,
    /// 最后检查结果
    pub last_result: Option<HealthCheckResult>,
    /// 连续失败次数
    pub consecutive_failures: u32,
    /// 是否启用
    pub enabled: bool,
}

/// 健康检查器
pub struct HealthChecker {
    /// 检查目标
    targets: Arc<DashMap<String, HealthCheckTarget>>,
    /// HTTP客户端
    http_client: reqwest::Client,
    /// 配置
    config: HealthCheckConfig,
    /// 是否运行中
    running: Arc<RwLock<bool>>,
}

impl HealthChecker {
    /// 创建新的健康检查器
    pub async fn new(config: HealthCheckConfig) -> ClusterResult<Self> {
        let http_client = reqwest::Client::builder()
            .timeout(config.default_timeout)
            .build()
            .map_err(|e| ClusterError::NetworkError(format!("创建HTTP客户端失败: {}", e)))?;
        
        info!("🏥 创建健康检查器");
        
        Ok(Self {
            targets: Arc::new(DashMap::new()),
            http_client,
            config,
            running: Arc::new(RwLock::new(false)),
        })
    }
    
    /// 启动健康检查器
    pub async fn start(&mut self) -> ClusterResult<()> {
        info!("🚀 启动健康检查器");
        
        {
            let mut running = self.running.write().await;
            *running = true;
        }
        
        // 启动健康检查任务
        self.start_health_check_task().await?;
        
        info!("✅ 健康检查器启动成功");
        Ok(())
    }
    
    /// 停止健康检查器
    pub async fn stop(&mut self) -> ClusterResult<()> {
        info!("🛑 停止健康检查器");
        
        {
            let mut running = self.running.write().await;
            *running = false;
        }
        
        info!("✅ 健康检查器已停止");
        Ok(())
    }
    
    /// 开始监控目标
    pub async fn start_monitoring(&self, target_id: &str, endpoint: String) -> ClusterResult<()> {
        let target = HealthCheckTarget {
            id: target_id.to_string(),
            endpoint,
            interval: self.config.default_interval,
            timeout: self.config.default_timeout,
            retries: self.config.default_retries,
            last_result: None,
            consecutive_failures: 0,
            enabled: true,
        };
        
        self.targets.insert(target_id.to_string(), target);
        
        debug!("开始监控目标: {}", target_id);
        Ok(())
    }
    
    /// 停止监控目标
    pub async fn stop_monitoring(&self, target_id: &str) -> ClusterResult<()> {
        self.targets.remove(target_id);
        
        debug!("停止监控目标: {}", target_id);
        Ok(())
    }
    
    /// 检查目标健康状态
    pub async fn check_health(&self, target_id: &str) -> ClusterResult<HealthStatus> {
        if let Some(target) = self.targets.get(target_id) {
            if let Some(ref result) = target.last_result {
                Ok(result.status.clone())
            } else {
                // 执行立即检查
                let result = self.perform_health_check(&target).await;
                match result {
                    Ok(check_result) => Ok(check_result.status),
                    Err(_) => Ok(HealthStatus::Unknown),
                }
            }
        } else {
            Ok(HealthStatus::Unknown)
        }
    }
    
    /// 获取健康检查结果
    pub async fn get_health_result(&self, target_id: &str) -> ClusterResult<Option<HealthCheckResult>> {
        if let Some(target) = self.targets.get(target_id) {
            Ok(target.last_result.clone())
        } else {
            Ok(None)
        }
    }
    
    /// 列出所有监控目标
    pub async fn list_targets(&self) -> ClusterResult<Vec<HealthCheckTarget>> {
        let targets: Vec<HealthCheckTarget> = self.targets.iter()
            .map(|entry| entry.value().clone())
            .collect();
        
        Ok(targets)
    }
    
    /// 更新目标配置
    pub async fn update_target_config(
        &self,
        target_id: &str,
        interval: Option<std::time::Duration>,
        timeout: Option<std::time::Duration>,
        retries: Option<u32>,
    ) -> ClusterResult<()> {
        if let Some(mut target) = self.targets.get_mut(target_id) {
            if let Some(interval) = interval {
                target.interval = interval;
            }
            if let Some(timeout) = timeout {
                target.timeout = timeout;
            }
            if let Some(retries) = retries {
                target.retries = retries;
            }
            
            debug!("更新目标配置: {}", target_id);
        }
        
        Ok(())
    }
    
    /// 启用/禁用目标监控
    pub async fn set_target_enabled(&self, target_id: &str, enabled: bool) -> ClusterResult<()> {
        if let Some(mut target) = self.targets.get_mut(target_id) {
            target.enabled = enabled;
            
            debug!("设置目标监控状态: {} -> {}", target_id, enabled);
        }
        
        Ok(())
    }
    
    /// 执行健康检查
    async fn perform_health_check(&self, target: &HealthCheckTarget) -> ClusterResult<HealthCheckResult> {
        let start_time = std::time::Instant::now();
        let checked_at = chrono::Utc::now();
        
        // 构建健康检查URL
        let health_url = if target.endpoint.contains("/health") {
            target.endpoint.clone()
        } else {
            format!("{}/health", target.endpoint.trim_end_matches('/'))
        };
        
        debug!("检查目标健康状态: {} -> {}", target.id, health_url);
        
        // 执行HTTP健康检查
        let mut retries = 0;
        let mut last_error = None;
        
        while retries <= target.retries {
            match self.http_client
                .get(&health_url)
                .timeout(target.timeout)
                .send()
                .await
            {
                Ok(response) => {
                    let response_time = start_time.elapsed().as_millis() as u64;
                    
                    if response.status().is_success() {
                        return Ok(HealthCheckResult {
                            target_id: target.id.clone(),
                            status: HealthStatus::Healthy,
                            response_time: Some(response_time),
                            error: None,
                            checked_at,
                            details: std::collections::HashMap::new(),
                        });
                    } else {
                        last_error = Some(format!("HTTP状态码: {}", response.status()));
                    }
                }
                Err(e) => {
                    last_error = Some(format!("请求失败: {}", e));
                }
            }
            
            retries += 1;
            if retries <= target.retries {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        }
        
        let response_time = start_time.elapsed().as_millis() as u64;
        
        Ok(HealthCheckResult {
            target_id: target.id.clone(),
            status: HealthStatus::Unhealthy,
            response_time: Some(response_time),
            error: last_error,
            checked_at,
            details: std::collections::HashMap::new(),
        })
    }
    
    /// 启动健康检查任务
    async fn start_health_check_task(&self) -> ClusterResult<()> {
        let targets = self.targets.clone();
        let http_client = self.http_client.clone();
        let running = self.running.clone();
        let check_interval = self.config.check_interval;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(check_interval);
            
            loop {
                interval.tick().await;
                
                // 检查是否还在运行
                {
                    let running = running.read().await;
                    if !*running {
                        break;
                    }
                }
                
                // 执行健康检查
                debug!("🏥 执行健康检查任务");
                
                let mut check_tasks = Vec::new();
                
                for entry in targets.iter() {
                    let target = entry.value().clone();
                    
                    if !target.enabled {
                        continue;
                    }
                    
                    // 检查是否到了检查时间
                    let should_check = if let Some(ref last_result) = target.last_result {
                        let elapsed = chrono::Utc::now()
                            .signed_duration_since(last_result.checked_at);
                        elapsed.to_std().unwrap_or_default() >= target.interval
                    } else {
                        true
                    };
                    
                    if should_check {
                        let target_clone = target.clone();
                        let client_clone = http_client.clone();
                        let targets_clone = targets.clone();
                        
                        let task = tokio::spawn(async move {
                            let checker = HealthChecker {
                                targets: targets_clone.clone(),
                                http_client: client_clone,
                                config: HealthCheckConfig::default(),
                                running: Arc::new(RwLock::new(true)),
                            };
                            
                            match checker.perform_health_check(&target_clone).await {
                                Ok(result) => {
                                    // 更新检查结果
                                    if let Some(mut target_entry) = targets_clone.get_mut(&target_clone.id) {
                                        // 更新连续失败次数
                                        if result.status == HealthStatus::Unhealthy {
                                            target_entry.consecutive_failures += 1;
                                        } else {
                                            target_entry.consecutive_failures = 0;
                                        }
                                        
                                        target_entry.last_result = Some(result.clone());
                                    }
                                    
                                    debug!("健康检查完成: {} -> {:?}", target_clone.id, result.status);
                                }
                                Err(e) => {
                                    warn!("健康检查失败: {} -> {}", target_clone.id, e);
                                }
                            }
                        });
                        
                        check_tasks.push(task);
                    }
                }
                
                // 等待所有检查任务完成
                for task in check_tasks {
                    let _ = task.await;
                }
            }
        });
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    
    #[tokio::test]
    async fn test_health_checker_creation() {
        let config = HealthCheckConfig::default();
        let health_checker = HealthChecker::new(config).await.unwrap();
        
        let targets = health_checker.list_targets().await.unwrap();
        assert_eq!(targets.len(), 0);
    }
    
    #[tokio::test]
    async fn test_target_management() {
        let config = HealthCheckConfig::default();
        let health_checker = HealthChecker::new(config).await.unwrap();
        
        // 开始监控目标
        health_checker.start_monitoring("test-target", "http://localhost:8080".to_string()).await.unwrap();
        
        // 列出目标
        let targets = health_checker.list_targets().await.unwrap();
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].id, "test-target");
        
        // 更新目标配置
        health_checker.update_target_config(
            "test-target",
            Some(Duration::from_secs(30)),
            Some(Duration::from_secs(10)),
            Some(3),
        ).await.unwrap();
        
        // 禁用目标监控
        health_checker.set_target_enabled("test-target", false).await.unwrap();
        
        let targets = health_checker.list_targets().await.unwrap();
        assert!(!targets[0].enabled);
        
        // 停止监控目标
        health_checker.stop_monitoring("test-target").await.unwrap();
        
        let targets = health_checker.list_targets().await.unwrap();
        assert_eq!(targets.len(), 0);
    }
    
    #[tokio::test]
    async fn test_health_status_check() {
        let config = HealthCheckConfig::default();
        let health_checker = HealthChecker::new(config).await.unwrap();
        
        // 添加一个不存在的目标
        health_checker.start_monitoring("nonexistent", "http://localhost:99999".to_string()).await.unwrap();
        
        // 检查健康状态（应该返回Unknown，因为还没有检查结果）
        let status = health_checker.check_health("nonexistent").await.unwrap();
        assert_eq!(status, HealthStatus::Unknown);
        
        // 检查不存在的目标
        let status = health_checker.check_health("not-found").await.unwrap();
        assert_eq!(status, HealthStatus::Unknown);
    }
}
