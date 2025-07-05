//! 路由器性能测试
//! 
//! 验证消息路由性能是否达到设计目标：
//! - 消息路由延迟 < 10ms
//! - 吞吐量 > 1000 msg/s

use agentx_router::*;
use agentx_a2a::{A2AMessage, AgentCard, AgentStatus, TrustLevel};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::timeout;

/// 创建测试路由器
fn create_test_router() -> MessageRouter {
    let strategy = Box::new(RoundRobinStrategy::new());
    let client = Arc::new(DefaultA2AClient);
    let cache = Arc::new(RouteCache::new(CacheConfig::default()));
    let metrics = Arc::new(RouterMetrics::new());
    let config = RouterConfig::default();
    
    MessageRouter::new(strategy, client, cache, metrics, config)
}

/// 创建测试Agent卡片
fn create_test_agent_card(id: &str) -> AgentCard {
    AgentCard {
        id: id.to_string(),
        name: format!("Test Agent {}", id),
        description: "Performance test agent".to_string(),
        version: "1.0.0".to_string(),
        capabilities: vec![],
        endpoints: vec![],
        metadata: HashMap::new(),
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
    }
}

/// 创建测试消息
fn create_test_message(target_agent: &str, content: &str) -> A2AMessage {
    let mut message = A2AMessage::user_message(content.to_string());
    message.metadata.insert(
        "target_agent".to_string(), 
        serde_json::Value::String(target_agent.to_string())
    );
    message
}

#[tokio::test]
async fn test_single_message_routing_latency() {
    println!("🚀 测试单消息路由延迟");
    
    let router = create_test_router();
    
    // 注册测试Agent
    let agent_card = create_test_agent_card("latency_test_agent");
    let endpoints = vec![
        AgentEndpoint::new("http://localhost:8090".to_string(), "http".to_string()),
    ];
    router.register_agent(agent_card, endpoints).await.unwrap();
    
    // 预热（避免首次调用的开销）
    for _ in 0..10 {
        let message = create_test_message("latency_test_agent", "warmup");
        let _ = router.route_message(message).await;
    }
    
    // 测试单消息路由延迟
    let mut latencies = Vec::new();
    
    for i in 0..100 {
        let message = create_test_message("latency_test_agent", &format!("test message {}", i));
        
        let start = Instant::now();
        let result = router.route_message(message).await;
        let latency = start.elapsed();
        
        assert!(result.is_ok(), "路由失败: {:?}", result.err());
        latencies.push(latency);
    }
    
    // 计算统计数据
    let avg_latency = latencies.iter().sum::<Duration>() / latencies.len() as u32;
    let max_latency = latencies.iter().max().unwrap();
    let min_latency = latencies.iter().min().unwrap();
    
    // 计算95th和99th百分位
    let mut sorted_latencies = latencies.clone();
    sorted_latencies.sort();
    let p95_index = (sorted_latencies.len() as f64 * 0.95) as usize;
    let p99_index = (sorted_latencies.len() as f64 * 0.99) as usize;
    let p95_latency = sorted_latencies[p95_index];
    let p99_latency = sorted_latencies[p99_index];
    
    println!("📊 延迟统计结果:");
    println!("   平均延迟: {:.2} ms", avg_latency.as_secs_f64() * 1000.0);
    println!("   最小延迟: {:.2} ms", min_latency.as_secs_f64() * 1000.0);
    println!("   最大延迟: {:.2} ms", max_latency.as_secs_f64() * 1000.0);
    println!("   95th百分位: {:.2} ms", p95_latency.as_secs_f64() * 1000.0);
    println!("   99th百分位: {:.2} ms", p99_latency.as_secs_f64() * 1000.0);
    
    // 验证性能目标：平均延迟 < 10ms
    let avg_latency_ms = avg_latency.as_secs_f64() * 1000.0;
    assert!(avg_latency_ms < 10.0, 
        "平均路由延迟 {:.2} ms 超过目标 10ms", avg_latency_ms);
    
    // 验证99th百分位延迟 < 50ms（更严格的要求）
    let p99_latency_ms = p99_latency.as_secs_f64() * 1000.0;
    assert!(p99_latency_ms < 50.0, 
        "99th百分位延迟 {:.2} ms 超过目标 50ms", p99_latency_ms);
    
    println!("✅ 延迟测试通过！");
}

#[tokio::test]
async fn test_throughput_performance() {
    println!("🚀 测试消息路由吞吐量");
    
    let router = create_test_router();
    
    // 注册多个测试Agent
    for i in 0..10 {
        let agent_card = create_test_agent_card(&format!("throughput_agent_{}", i));
        let endpoints = vec![
            AgentEndpoint::new(format!("http://localhost:809{}", i), "http".to_string()),
        ];
        router.register_agent(agent_card, endpoints).await.unwrap();
    }
    
    // 测试吞吐量
    let test_duration = Duration::from_secs(5); // 5秒测试
    let start_time = Instant::now();
    let mut message_count = 0;
    let mut successful_routes = 0;
    
    println!("📈 开始吞吐量测试 ({}秒)...", test_duration.as_secs());
    
    while start_time.elapsed() < test_duration {
        let agent_id = format!("throughput_agent_{}", message_count % 10);
        let message = create_test_message(&agent_id, &format!("throughput test {}", message_count));
        
        match timeout(Duration::from_millis(100), router.route_message(message)).await {
            Ok(Ok(_)) => {
                successful_routes += 1;
            }
            Ok(Err(e)) => {
                eprintln!("路由失败: {:?}", e);
            }
            Err(_) => {
                eprintln!("路由超时");
            }
        }
        
        message_count += 1;
    }
    
    let actual_duration = start_time.elapsed();
    let throughput = successful_routes as f64 / actual_duration.as_secs_f64();
    
    println!("📊 吞吐量测试结果:");
    println!("   测试时长: {:.2} 秒", actual_duration.as_secs_f64());
    println!("   总消息数: {}", message_count);
    println!("   成功路由: {}", successful_routes);
    println!("   成功率: {:.2}%", (successful_routes as f64 / message_count as f64) * 100.0);
    println!("   吞吐量: {:.2} msg/s", throughput);
    
    // 验证性能目标：吞吐量 > 1000 msg/s
    assert!(throughput > 1000.0, 
        "吞吐量 {:.2} msg/s 低于目标 1000 msg/s", throughput);
    
    // 验证成功率 > 95%
    let success_rate = (successful_routes as f64 / message_count as f64) * 100.0;
    assert!(success_rate > 95.0, 
        "成功率 {:.2}% 低于目标 95%", success_rate);
    
    println!("✅ 吞吐量测试通过！");
}

#[tokio::test]
async fn test_concurrent_routing_performance() {
    println!("🚀 测试并发路由性能");
    
    let router = Arc::new(create_test_router());
    
    // 注册测试Agent
    for i in 0..20 {
        let agent_card = create_test_agent_card(&format!("concurrent_agent_{}", i));
        let endpoints = vec![
            AgentEndpoint::new(format!("http://localhost:810{}", i), "http".to_string()),
        ];
        router.register_agent(agent_card, endpoints).await.unwrap();
    }
    
    // 并发测试
    let concurrent_tasks = 50;
    let messages_per_task = 20;
    
    println!("📈 开始并发测试 ({} 任务, 每任务 {} 消息)...", 
             concurrent_tasks, messages_per_task);
    
    let start_time = Instant::now();
    let mut handles = Vec::new();
    
    for task_id in 0..concurrent_tasks {
        let router_clone = router.clone();
        let handle = tokio::spawn(async move {
            let mut successful = 0;
            let mut failed = 0;
            
            for msg_id in 0..messages_per_task {
                let agent_id = format!("concurrent_agent_{}", (task_id + msg_id) % 20);
                let message = create_test_message(&agent_id, 
                    &format!("concurrent test task:{} msg:{}", task_id, msg_id));
                
                match router_clone.route_message(message).await {
                    Ok(_) => successful += 1,
                    Err(_) => failed += 1,
                }
            }
            
            (successful, failed)
        });
        handles.push(handle);
    }
    
    // 等待所有任务完成
    let mut total_successful = 0;
    let mut total_failed = 0;
    
    for handle in handles {
        let (successful, failed) = handle.await.unwrap();
        total_successful += successful;
        total_failed += failed;
    }
    
    let total_duration = start_time.elapsed();
    let total_messages = concurrent_tasks * messages_per_task;
    let throughput = total_successful as f64 / total_duration.as_secs_f64();
    
    println!("📊 并发测试结果:");
    println!("   测试时长: {:.2} 秒", total_duration.as_secs_f64());
    println!("   总消息数: {}", total_messages);
    println!("   成功路由: {}", total_successful);
    println!("   失败路由: {}", total_failed);
    println!("   成功率: {:.2}%", (total_successful as f64 / total_messages as f64) * 100.0);
    println!("   并发吞吐量: {:.2} msg/s", throughput);
    
    // 验证并发性能
    assert!(throughput > 500.0, 
        "并发吞吐量 {:.2} msg/s 低于目标 500 msg/s", throughput);
    
    let success_rate = (total_successful as f64 / total_messages as f64) * 100.0;
    assert!(success_rate > 90.0, 
        "并发成功率 {:.2}% 低于目标 90%", success_rate);
    
    println!("✅ 并发测试通过！");
}

#[tokio::test]
async fn test_cache_performance_impact() {
    println!("🚀 测试缓存对性能的影响");
    
    let router = create_test_router();
    
    // 注册测试Agent
    let agent_card = create_test_agent_card("cache_test_agent");
    let endpoints = vec![
        AgentEndpoint::new("http://localhost:8095".to_string(), "http".to_string()),
    ];
    router.register_agent(agent_card, endpoints).await.unwrap();
    
    // 测试缓存未命中的性能
    let mut cold_latencies = Vec::new();
    for i in 0..50 {
        // 使用不同的目标确保缓存未命中
        let agent_id = format!("cache_test_agent_{}", i);
        let agent_card = create_test_agent_card(&agent_id);
        let endpoints = vec![
            AgentEndpoint::new(format!("http://localhost:820{}", i), "http".to_string()),
        ];
        router.register_agent(agent_card, endpoints).await.unwrap();
        
        let message = create_test_message(&agent_id, "cache miss test");
        
        let start = Instant::now();
        let _ = router.route_message(message).await;
        cold_latencies.push(start.elapsed());
    }
    
    // 测试缓存命中的性能
    let mut warm_latencies = Vec::new();
    for _ in 0..50 {
        let message = create_test_message("cache_test_agent", "cache hit test");
        
        let start = Instant::now();
        let _ = router.route_message(message).await;
        warm_latencies.push(start.elapsed());
    }
    
    let cold_avg = cold_latencies.iter().sum::<Duration>() / cold_latencies.len() as u32;
    let warm_avg = warm_latencies.iter().sum::<Duration>() / warm_latencies.len() as u32;
    
    println!("📊 缓存性能影响:");
    println!("   缓存未命中平均延迟: {:.2} ms", cold_avg.as_secs_f64() * 1000.0);
    println!("   缓存命中平均延迟: {:.2} ms", warm_avg.as_secs_f64() * 1000.0);
    println!("   性能提升: {:.2}x", cold_avg.as_secs_f64() / warm_avg.as_secs_f64());
    
    // 验证缓存确实提升了性能
    assert!(warm_avg < cold_avg, "缓存没有提升性能");
    
    // 验证缓存命中延迟仍然很低
    let warm_avg_ms = warm_avg.as_secs_f64() * 1000.0;
    assert!(warm_avg_ms < 5.0, 
        "缓存命中延迟 {:.2} ms 仍然过高", warm_avg_ms);
    
    println!("✅ 缓存性能测试通过！");
}
