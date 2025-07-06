//! 路由缓存实现
//! 
//! 提供Agent信息和路由结果的缓存功能，提高路由性能

use crate::AgentInfo;
use agentx_a2a::AgentCard;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, warn};

/// 缓存条目
#[derive(Debug, Clone)]
struct CacheEntry<T> {
    /// 缓存的值
    value: T,
    /// 创建时间
    #[allow(dead_code)]
    created_at: Instant,
    /// 过期时间
    expires_at: Instant,
    /// 访问次数
    access_count: u64,
    /// 最后访问时间
    last_accessed: Instant,
}

impl<T> CacheEntry<T> {
    fn new(value: T, ttl: Duration) -> Self {
        let now = Instant::now();
        Self {
            value,
            created_at: now,
            expires_at: now + ttl,
            access_count: 0,
            last_accessed: now,
        }
    }

    fn is_expired(&self) -> bool {
        Instant::now() > self.expires_at
    }

    fn access(&mut self) -> &T {
        self.access_count += 1;
        self.last_accessed = Instant::now();
        &self.value
    }
}

/// 路由缓存
#[derive(Debug)]
pub struct RouteCache {
    /// Agent信息缓存
    agent_cache: Arc<RwLock<HashMap<String, CacheEntry<AgentInfo>>>>,
    /// Agent卡片缓存
    card_cache: Arc<RwLock<HashMap<String, CacheEntry<AgentCard>>>>,
    /// 路由结果缓存
    route_cache: Arc<RwLock<HashMap<String, CacheEntry<String>>>>,
    /// 缓存配置
    config: CacheConfig,
}

/// 缓存配置
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Agent信息TTL
    pub agent_ttl: Duration,
    /// Agent卡片TTL
    pub card_ttl: Duration,
    /// 路由结果TTL
    pub route_ttl: Duration,
    /// 最大缓存大小
    pub max_size: usize,
    /// 清理间隔
    pub cleanup_interval: Duration,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            agent_ttl: Duration::from_secs(300),      // 5分钟
            card_ttl: Duration::from_secs(600),       // 10分钟
            route_ttl: Duration::from_secs(60),       // 1分钟
            max_size: 10000,                          // 最大10000条记录
            cleanup_interval: Duration::from_secs(60), // 1分钟清理一次
        }
    }
}

impl RouteCache {
    /// 创建新的路由缓存
    pub fn new(config: CacheConfig) -> Self {
        Self {
            agent_cache: Arc::new(RwLock::new(HashMap::new())),
            card_cache: Arc::new(RwLock::new(HashMap::new())),
            route_cache: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// 缓存Agent信息
    pub async fn cache_agent(&self, agent_id: String, agent_info: AgentInfo) {
        let entry = CacheEntry::new(agent_info, self.config.agent_ttl);
        let mut cache = self.agent_cache.write().await;
        
        // 检查缓存大小限制
        if cache.len() >= self.config.max_size {
            self.evict_oldest_agent(&mut cache).await;
        }
        
        cache.insert(agent_id.clone(), entry);
        debug!("缓存Agent信息: {}", agent_id);
    }

    /// 获取缓存的Agent信息
    pub async fn get_agent(&self, agent_id: &str) -> Option<AgentInfo> {
        let mut cache = self.agent_cache.write().await;
        
        if let Some(entry) = cache.get_mut(agent_id) {
            if entry.is_expired() {
                cache.remove(agent_id);
                debug!("Agent缓存已过期: {}", agent_id);
                None
            } else {
                debug!("Agent缓存命中: {}", agent_id);
                Some(entry.access().clone())
            }
        } else {
            debug!("Agent缓存未命中: {}", agent_id);
            None
        }
    }

    /// 缓存Agent卡片
    pub async fn cache_card(&self, agent_id: String, card: AgentCard) {
        let entry = CacheEntry::new(card, self.config.card_ttl);
        let mut cache = self.card_cache.write().await;
        
        // 检查缓存大小限制
        if cache.len() >= self.config.max_size {
            self.evict_oldest_card(&mut cache).await;
        }
        
        cache.insert(agent_id.clone(), entry);
        debug!("缓存Agent卡片: {}", agent_id);
    }

    /// 获取缓存的Agent卡片
    pub async fn get_card(&self, agent_id: &str) -> Option<AgentCard> {
        let mut cache = self.card_cache.write().await;
        
        if let Some(entry) = cache.get_mut(agent_id) {
            if entry.is_expired() {
                cache.remove(agent_id);
                debug!("Agent卡片缓存已过期: {}", agent_id);
                None
            } else {
                debug!("Agent卡片缓存命中: {}", agent_id);
                Some(entry.access().clone())
            }
        } else {
            debug!("Agent卡片缓存未命中: {}", agent_id);
            None
        }
    }

    /// 缓存路由结果
    pub async fn cache_route(&self, route_key: String, target_agent: String) {
        let entry = CacheEntry::new(target_agent, self.config.route_ttl);
        let mut cache = self.route_cache.write().await;
        
        // 检查缓存大小限制
        if cache.len() >= self.config.max_size {
            self.evict_oldest_route(&mut cache).await;
        }
        
        cache.insert(route_key.clone(), entry);
        debug!("缓存路由结果: {}", route_key);
    }

    /// 获取缓存的路由结果
    pub async fn get_route(&self, route_key: &str) -> Option<String> {
        let mut cache = self.route_cache.write().await;
        
        if let Some(entry) = cache.get_mut(route_key) {
            if entry.is_expired() {
                cache.remove(route_key);
                debug!("路由缓存已过期: {}", route_key);
                None
            } else {
                debug!("路由缓存命中: {}", route_key);
                Some(entry.access().clone())
            }
        } else {
            debug!("路由缓存未命中: {}", route_key);
            None
        }
    }

    /// 失效Agent缓存
    pub async fn invalidate_agent(&self, agent_id: &str) {
        self.agent_cache.write().await.remove(agent_id);
        debug!("失效Agent缓存: {}", agent_id);
    }

    /// 失效Agent卡片缓存
    pub async fn invalidate_card(&self, agent_id: &str) {
        self.card_cache.write().await.remove(agent_id);
        debug!("失效Agent卡片缓存: {}", agent_id);
    }

    /// 失效路由缓存
    pub async fn invalidate_route(&self, route_key: &str) {
        self.route_cache.write().await.remove(route_key);
        debug!("失效路由缓存: {}", route_key);
    }

    /// 清理过期缓存
    pub async fn cleanup_expired(&self) {
        let _now = Instant::now();
        let mut removed_count = 0;

        // 清理Agent缓存
        {
            let mut cache = self.agent_cache.write().await;
            let before_size = cache.len();
            cache.retain(|_, entry| !entry.is_expired());
            removed_count += before_size - cache.len();
        }

        // 清理Agent卡片缓存
        {
            let mut cache = self.card_cache.write().await;
            let before_size = cache.len();
            cache.retain(|_, entry| !entry.is_expired());
            removed_count += before_size - cache.len();
        }

        // 清理路由缓存
        {
            let mut cache = self.route_cache.write().await;
            let before_size = cache.len();
            cache.retain(|_, entry| !entry.is_expired());
            removed_count += before_size - cache.len();
        }

        if removed_count > 0 {
            debug!("清理过期缓存: {} 条记录", removed_count);
        }
    }

    /// 获取缓存统计
    pub async fn get_stats(&self) -> CacheStats {
        let agent_count = self.agent_cache.read().await.len();
        let card_count = self.card_cache.read().await.len();
        let route_count = self.route_cache.read().await.len();

        CacheStats {
            agent_cache_size: agent_count,
            card_cache_size: card_count,
            route_cache_size: route_count,
            total_size: agent_count + card_count + route_count,
            max_size: self.config.max_size,
        }
    }

    /// 清空所有缓存
    pub async fn clear_all(&self) {
        self.agent_cache.write().await.clear();
        self.card_cache.write().await.clear();
        self.route_cache.write().await.clear();
        debug!("清空所有缓存");
    }

    // 私有方法

    async fn evict_oldest_agent(&self, cache: &mut HashMap<String, CacheEntry<AgentInfo>>) {
        if let Some((oldest_key, _)) = cache
            .iter()
            .min_by_key(|(_, entry)| entry.last_accessed)
            .map(|(k, v)| (k.clone(), v.clone()))
        {
            cache.remove(&oldest_key);
            warn!("驱逐最旧的Agent缓存: {}", oldest_key);
        }
    }

    async fn evict_oldest_card(&self, cache: &mut HashMap<String, CacheEntry<AgentCard>>) {
        if let Some((oldest_key, _)) = cache
            .iter()
            .min_by_key(|(_, entry)| entry.last_accessed)
            .map(|(k, v)| (k.clone(), v.clone()))
        {
            cache.remove(&oldest_key);
            warn!("驱逐最旧的Agent卡片缓存: {}", oldest_key);
        }
    }

    async fn evict_oldest_route(&self, cache: &mut HashMap<String, CacheEntry<String>>) {
        if let Some((oldest_key, _)) = cache
            .iter()
            .min_by_key(|(_, entry)| entry.last_accessed)
            .map(|(k, v)| (k.clone(), v.clone()))
        {
            cache.remove(&oldest_key);
            warn!("驱逐最旧的路由缓存: {}", oldest_key);
        }
    }
}

/// 缓存统计信息
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub agent_cache_size: usize,
    pub card_cache_size: usize,
    pub route_cache_size: usize,
    pub total_size: usize,
    pub max_size: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use agentx_a2a::{AgentStatus, TrustLevel};

    #[tokio::test]
    async fn test_agent_cache() {
        let config = CacheConfig::default();
        let cache = RouteCache::new(config);

        // 创建测试Agent信息
        let agent_card = AgentCard {
            id: "test_agent".to_string(),
            name: "Test Agent".to_string(),
            description: "Test Description".to_string(),
            version: "1.0.0".to_string(),
            capabilities: vec![],
            endpoints: vec![],
            metadata: std::collections::HashMap::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            expires_at: None,
            status: AgentStatus::Online,
            supported_versions: vec!["1.0.0".to_string()],
            tags: vec![],
            interaction_modalities: vec![],
            ux_capabilities: None,
            trust_level: TrustLevel::Public,
            supported_task_types: vec![],
        };

        let agent_info = AgentInfo {
            card: agent_card.clone(),
            endpoints: vec![],
            load: 0.5,
            response_time: crate::ResponseTimeStats {
                average_ms: 100.0,
                p95_ms: 150.0,
                p99_ms: 200.0,
                sample_count: 10,
            },
            last_updated: chrono::Utc::now(),
            health: crate::HealthStatus::Healthy,
        };

        // 测试缓存和获取
        cache.cache_agent("test_agent".to_string(), agent_info.clone()).await;
        let cached = cache.get_agent("test_agent").await;
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().card.id, "test_agent");

        // 测试缓存未命中
        let not_found = cache.get_agent("non_existent").await;
        assert!(not_found.is_none());
    }
}
