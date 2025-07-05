//! 路由器指标收集
//! 
//! 提供路由器性能监控和指标收集功能

use crate::ErrorStats;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};

/// 路由器指标收集器
#[derive(Debug)]
pub struct RouterMetrics {
    /// 路由统计
    route_stats: Arc<RwLock<RouteStats>>,
    /// Agent统计
    agent_stats: Arc<RwLock<HashMap<String, AgentStats>>>,
    /// 错误统计
    error_stats: Arc<RwLock<ErrorStats>>,
    /// 缓存统计
    cache_stats: Arc<RwLock<CacheStats>>,
}

/// 路由统计信息
#[derive(Debug, Clone, Default)]
pub struct RouteStats {
    /// 总路由请求数
    pub total_requests: u64,
    /// 成功路由数
    pub successful_routes: u64,
    /// 失败路由数
    pub failed_routes: u64,
    /// 平均路由时间（毫秒）
    pub avg_routing_time_ms: f64,
    /// 最大路由时间（毫秒）
    pub max_routing_time_ms: u64,
    /// 最小路由时间（毫秒）
    pub min_routing_time_ms: u64,
    /// 总路由时间（用于计算平均值）
    pub total_routing_time_ms: u64,
    /// 最后更新时间
    pub last_updated: DateTime<Utc>,
}

/// Agent统计信息
#[derive(Debug, Clone, Default)]
pub struct AgentStats {
    /// Agent ID
    pub agent_id: String,
    /// 总请求数
    pub total_requests: u64,
    /// 成功请求数
    pub successful_requests: u64,
    /// 失败请求数
    pub failed_requests: u64,
    /// 平均响应时间（毫秒）
    pub avg_response_time_ms: f64,
    /// 最大响应时间（毫秒）
    pub max_response_time_ms: u64,
    /// 最小响应时间（毫秒）
    pub min_response_time_ms: u64,
    /// 总响应时间（用于计算平均值）
    pub total_response_time_ms: u64,
    /// 当前负载
    pub current_load: f64,
    /// 最后请求时间
    pub last_request_time: DateTime<Utc>,
}

/// 缓存统计信息
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    /// 缓存命中数
    pub cache_hits: u64,
    /// 缓存未命中数
    pub cache_misses: u64,
    /// 缓存失效数
    pub cache_invalidations: u64,
    /// 缓存大小
    pub cache_size: usize,
    /// 最后更新时间
    pub last_updated: DateTime<Utc>,
}

impl RouterMetrics {
    /// 创建新的指标收集器
    pub fn new() -> Self {
        Self {
            route_stats: Arc::new(RwLock::new(RouteStats::default())),
            agent_stats: Arc::new(RwLock::new(HashMap::new())),
            error_stats: Arc::new(RwLock::new(ErrorStats::default())),
            cache_stats: Arc::new(RwLock::new(CacheStats::default())),
        }
    }

    /// 记录成功路由
    pub async fn record_successful_route(&self, agent_id: &str, response_time_ms: u64) {
        // 更新路由统计
        {
            let mut stats = self.route_stats.write().await;
            stats.total_requests += 1;
            stats.successful_routes += 1;
            stats.total_routing_time_ms += response_time_ms;
            stats.avg_routing_time_ms = stats.total_routing_time_ms as f64 / stats.total_requests as f64;
            
            if response_time_ms > stats.max_routing_time_ms {
                stats.max_routing_time_ms = response_time_ms;
            }
            
            if stats.min_routing_time_ms == 0 || response_time_ms < stats.min_routing_time_ms {
                stats.min_routing_time_ms = response_time_ms;
            }
            
            stats.last_updated = Utc::now();
        }

        // 更新Agent统计
        {
            let mut agent_stats = self.agent_stats.write().await;
            let stats = agent_stats.entry(agent_id.to_string()).or_insert_with(|| AgentStats {
                agent_id: agent_id.to_string(),
                min_response_time_ms: u64::MAX,
                ..Default::default()
            });
            
            stats.total_requests += 1;
            stats.successful_requests += 1;
            stats.total_response_time_ms += response_time_ms;
            stats.avg_response_time_ms = stats.total_response_time_ms as f64 / stats.total_requests as f64;
            
            if response_time_ms > stats.max_response_time_ms {
                stats.max_response_time_ms = response_time_ms;
            }
            
            if response_time_ms < stats.min_response_time_ms {
                stats.min_response_time_ms = response_time_ms;
            }
            
            stats.last_request_time = Utc::now();
        }
    }

    /// 记录失败路由
    pub async fn record_failed_route(&self, agent_id: &str) {
        // 更新路由统计
        {
            let mut stats = self.route_stats.write().await;
            stats.total_requests += 1;
            stats.failed_routes += 1;
            stats.last_updated = Utc::now();
        }

        // 更新Agent统计
        {
            let mut agent_stats = self.agent_stats.write().await;
            let stats = agent_stats.entry(agent_id.to_string()).or_insert_with(|| AgentStats {
                agent_id: agent_id.to_string(),
                min_response_time_ms: u64::MAX,
                ..Default::default()
            });
            
            stats.total_requests += 1;
            stats.failed_requests += 1;
            stats.last_request_time = Utc::now();
        }
    }

    /// 记录缓存命中
    pub async fn record_cache_hit(&self) {
        let mut stats = self.cache_stats.write().await;
        stats.cache_hits += 1;
        stats.last_updated = Utc::now();
    }

    /// 记录缓存未命中
    pub async fn record_cache_miss(&self) {
        let mut stats = self.cache_stats.write().await;
        stats.cache_misses += 1;
        stats.last_updated = Utc::now();
    }

    /// 记录缓存失效
    pub async fn record_cache_invalidation(&self) {
        let mut stats = self.cache_stats.write().await;
        stats.cache_invalidations += 1;
        stats.last_updated = Utc::now();
    }

    /// 更新缓存大小
    pub async fn update_cache_size(&self, size: usize) {
        let mut stats = self.cache_stats.write().await;
        stats.cache_size = size;
        stats.last_updated = Utc::now();
    }

    /// 更新Agent负载
    pub async fn update_agent_load(&self, agent_id: &str, load: f64) {
        let mut agent_stats = self.agent_stats.write().await;
        let stats = agent_stats.entry(agent_id.to_string()).or_insert_with(|| AgentStats {
            agent_id: agent_id.to_string(),
            min_response_time_ms: u64::MAX,
            ..Default::default()
        });
        
        stats.current_load = load;
    }

    /// 获取路由统计
    pub async fn get_route_stats(&self) -> RouteStats {
        self.route_stats.read().await.clone()
    }

    /// 获取Agent统计
    pub async fn get_agent_stats(&self, agent_id: &str) -> Option<AgentStats> {
        self.agent_stats.read().await.get(agent_id).cloned()
    }

    /// 获取所有Agent统计
    pub async fn get_all_agent_stats(&self) -> HashMap<String, AgentStats> {
        self.agent_stats.read().await.clone()
    }

    /// 获取缓存统计
    pub async fn get_cache_stats(&self) -> CacheStats {
        self.cache_stats.read().await.clone()
    }

    /// 获取错误统计
    pub async fn get_error_stats(&self) -> ErrorStats {
        self.error_stats.read().await.clone()
    }

    /// 计算成功率
    pub async fn get_success_rate(&self) -> f64 {
        let stats = self.route_stats.read().await;
        if stats.total_requests == 0 {
            0.0
        } else {
            stats.successful_routes as f64 / stats.total_requests as f64
        }
    }

    /// 计算缓存命中率
    pub async fn get_cache_hit_rate(&self) -> f64 {
        let stats = self.cache_stats.read().await;
        let total = stats.cache_hits + stats.cache_misses;
        if total == 0 {
            0.0
        } else {
            stats.cache_hits as f64 / total as f64
        }
    }

    /// 获取性能摘要
    pub async fn get_performance_summary(&self) -> PerformanceSummary {
        let route_stats = self.get_route_stats().await;
        let cache_stats = self.get_cache_stats().await;
        let success_rate = self.get_success_rate().await;
        let cache_hit_rate = self.get_cache_hit_rate().await;

        PerformanceSummary {
            total_requests: route_stats.total_requests,
            successful_routes: route_stats.successful_routes,
            failed_routes: route_stats.failed_routes,
            success_rate,
            avg_routing_time_ms: route_stats.avg_routing_time_ms,
            max_routing_time_ms: route_stats.max_routing_time_ms,
            min_routing_time_ms: route_stats.min_routing_time_ms,
            cache_hit_rate,
            cache_hits: cache_stats.cache_hits,
            cache_misses: cache_stats.cache_misses,
            active_agents: self.agent_stats.read().await.len(),
        }
    }

    /// 重置所有统计
    pub async fn reset_stats(&self) {
        *self.route_stats.write().await = RouteStats::default();
        self.agent_stats.write().await.clear();
        *self.error_stats.write().await = ErrorStats::default();
        *self.cache_stats.write().await = CacheStats::default();
    }
}

/// 性能摘要
#[derive(Debug, Clone)]
pub struct PerformanceSummary {
    pub total_requests: u64,
    pub successful_routes: u64,
    pub failed_routes: u64,
    pub success_rate: f64,
    pub avg_routing_time_ms: f64,
    pub max_routing_time_ms: u64,
    pub min_routing_time_ms: u64,
    pub cache_hit_rate: f64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub active_agents: usize,
}

impl Default for RouterMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_route_metrics() {
        let metrics = RouterMetrics::new();
        
        // 记录成功路由
        metrics.record_successful_route("agent1", 100).await;
        metrics.record_successful_route("agent1", 200).await;
        
        let stats = metrics.get_route_stats().await;
        assert_eq!(stats.total_requests, 2);
        assert_eq!(stats.successful_routes, 2);
        assert_eq!(stats.avg_routing_time_ms, 150.0);
        
        let success_rate = metrics.get_success_rate().await;
        assert_eq!(success_rate, 1.0);
    }

    #[tokio::test]
    async fn test_cache_metrics() {
        let metrics = RouterMetrics::new();
        
        metrics.record_cache_hit().await;
        metrics.record_cache_hit().await;
        metrics.record_cache_miss().await;
        
        let hit_rate = metrics.get_cache_hit_rate().await;
        assert!((hit_rate - 0.6666666666666666).abs() < f64::EPSILON);
    }
}
