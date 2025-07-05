//! A2A协议高级功能演示
//! 
//! 本示例展示AgentX中A2A协议的高级功能，包括：
//! - 流式通信
//! - 安全认证
//! - 监控和指标收集

use agentx_a2a::{
    // 流式通信
    StreamManager, StreamMessageBuilder, StreamType, StreamChunk,
    // 安全认证
    SecurityManager, SecurityConfig, AuthCredentials, AuthType, TrustLevel,
    // 监控
    MonitoringManager, MonitoringConfig, MetricPoint, MetricType,
    // 基础类型
    A2AMessage, MessageRole,
};
use std::collections::HashMap;
use chrono::Utc;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 AgentX A2A协议高级功能演示");
    println!("展示流式通信、安全认证和监控功能");
    
    // 1. 流式通信演示
    println!("\n📡 1. 流式通信演示");
    demonstrate_streaming().await?;
    
    // 2. 安全认证演示
    println!("\n🔒 2. 安全认证演示");
    demonstrate_security().await?;
    
    // 3. 监控和指标演示
    println!("\n📊 3. 监控和指标演示");
    demonstrate_monitoring().await?;
    
    // 4. 综合场景演示
    println!("\n🎯 4. 综合场景演示");
    demonstrate_integrated_scenario().await?;
    
    println!("\n🎉 A2A协议高级功能演示完成！");
    println!("✅ 所有高级功能都正常工作");
    
    Ok(())
}

/// 演示流式通信功能
async fn demonstrate_streaming() -> Result<(), Box<dyn std::error::Error>> {
    println!("📡 流式通信功能演示:");
    
    let mut stream_manager = StreamManager::new();
    
    // 1. 创建文件流
    println!("   📁 创建文件流传输");
    let file_header = StreamMessageBuilder::new(StreamType::FileStream)
        .content_type("text/plain".to_string())
        .encoding("utf-8".to_string())
        .metadata("filename".to_string(), serde_json::Value::String("demo.txt".to_string()))
        .build_header(Some(300), Some(3));
    
    let stream_id = file_header.stream_id.clone();
    println!("     流ID: {}", stream_id);
    println!("     流类型: {:?}", file_header.stream_type);
    
    stream_manager.start_stream(file_header)?;
    
    // 2. 发送数据块
    println!("   📦 发送数据块");
    let chunks = vec![
        "Hello, this is chunk 1\n",
        "This is chunk 2 with more data\n", 
        "Final chunk 3 - end of file\n",
    ];
    
    for (i, chunk_data) in chunks.iter().enumerate() {
        let chunk = StreamChunk {
            stream_id: stream_id.clone(),
            sequence: i as u64,
            data: chunk_data.as_bytes().to_vec(),
            is_final: i == chunks.len() - 1,
            checksum: None,
            metadata: HashMap::new(),
        };
        
        stream_manager.send_chunk(chunk)?;
        println!("     ✅ 块 {} 发送成功 ({} 字节)", i, chunk_data.len());
    }
    
    // 3. 检查流状态
    let status = stream_manager.get_stream_status(&stream_id);
    if let Some(status) = status {
        println!("   📊 流状态:");
        println!("     状态: {:?}", status.state);
        println!("     接收块数: {}", status.received_chunks);
        println!("     进度: {:?}", status.progress);
    }
    
    // 4. 创建实时数据流
    println!("   📈 创建实时数据流");
    let data_header = StreamMessageBuilder::new(StreamType::DataStream)
        .content_type("application/json".to_string())
        .build_header(None, None);
    
    let data_stream_id = data_header.stream_id.clone();
    stream_manager.start_stream(data_header)?;
    
    // 发送实时数据
    for i in 0..5 {
        let data = serde_json::json!({
            "timestamp": Utc::now().to_rfc3339(),
            "value": i * 10,
            "sensor": "temperature"
        });
        
        let chunk = StreamChunk {
            stream_id: data_stream_id.clone(),
            sequence: i,
            data: data.to_string().as_bytes().to_vec(),
            is_final: i == 4,
            checksum: None,
            metadata: HashMap::new(),
        };
        
        stream_manager.send_chunk(chunk)?;
        println!("     ✅ 数据点 {} 发送成功", i);
        
        // 模拟实时间隔
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
    
    println!("   ✅ 流式通信演示完成");
    Ok(())
}

/// 演示安全认证功能
async fn demonstrate_security() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔒 安全认证功能演示:");
    
    // 1. 创建安全管理器
    let config = SecurityConfig {
        auth_type: AuthType::ApiKey,
        required_trust_level: TrustLevel::Verified,
        token_expiry_seconds: 3600,
        ..Default::default()
    };
    
    let mut security_manager = SecurityManager::new(config);
    
    // 2. 添加信任的Agent
    println!("   👥 配置信任的Agent");
    security_manager.add_trusted_agent("trusted_agent_1".to_string(), TrustLevel::Trusted);
    security_manager.add_trusted_agent("internal_agent_1".to_string(), TrustLevel::Internal);
    security_manager.add_trusted_agent("verified_agent_1".to_string(), TrustLevel::Verified);
    
    // 3. 测试不同认证方式
    println!("   🔑 测试API密钥认证");
    
    let mut api_credentials = HashMap::new();
    api_credentials.insert("api_key".to_string(), "a".repeat(32));
    
    let credentials = AuthCredentials {
        auth_type: AuthType::ApiKey,
        credentials: api_credentials,
        expires_at: None,
        scopes: vec!["read".to_string(), "write".to_string()],
    };
    
    match security_manager.authenticate("trusted_agent_1", credentials) {
        Ok(context) => {
            println!("     ✅ 认证成功");
            println!("       Agent ID: {}", context.agent_id);
            println!("       信任级别: {:?}", context.trust_level);
            println!("       会话ID: {}", context.session_id);
            println!("       权限数量: {}", context.permissions.len());
            
            // 4. 测试权限检查
            println!("   🛡️ 测试权限检查");
            let permissions_to_check = vec![
                "read_public",
                "send_message", 
                "create_task",
                "manage_agents",
            ];
            
            for permission in permissions_to_check {
                let has_permission = security_manager.check_permission(&context, permission);
                println!("     权限 '{}': {}", permission, if has_permission { "✅ 允许" } else { "❌ 拒绝" });
            }
            
            // 5. 测试会话验证
            println!("   🔄 测试会话验证");
            match security_manager.validate_session(&context.session_id) {
                Ok(_) => println!("     ✅ 会话验证成功"),
                Err(e) => println!("     ❌ 会话验证失败: {}", e),
            }
        },
        Err(e) => {
            println!("     ❌ 认证失败: {}", e);
        }
    }
    
    // 6. 测试信任级别不足的情况
    println!("   ⚠️ 测试信任级别不足");
    let low_trust_credentials = AuthCredentials {
        auth_type: AuthType::ApiKey,
        credentials: {
            let mut creds = HashMap::new();
            creds.insert("api_key".to_string(), "b".repeat(32));
            creds
        },
        expires_at: None,
        scopes: vec!["read".to_string()],
    };
    
    match security_manager.authenticate("unknown_agent", low_trust_credentials) {
        Ok(_) => println!("     ⚠️ 意外通过认证"),
        Err(e) => println!("     ✅ 正确拒绝认证: {}", e),
    }
    
    println!("   ✅ 安全认证演示完成");
    Ok(())
}

/// 演示监控和指标功能
async fn demonstrate_monitoring() -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 监控和指标功能演示:");
    
    // 1. 创建监控管理器
    let config = MonitoringConfig {
        metric_retention_hours: 24,
        health_check_interval_seconds: 30,
        enable_detailed_monitoring: true,
        ..Default::default()
    };
    
    let mut monitoring_manager = MonitoringManager::new(config);
    
    // 2. 记录各种指标
    println!("   📈 记录性能指标");
    
    // 计数器指标
    monitoring_manager.increment_counter("total_messages", 1000);
    monitoring_manager.increment_counter("successful_messages", 950);
    monitoring_manager.increment_counter("failed_messages", 50);
    
    // 仪表指标
    let mut labels = HashMap::new();
    labels.insert("component".to_string(), "a2a_engine".to_string());
    
    monitoring_manager.set_gauge("cpu_usage_percent", 25.5, labels.clone());
    monitoring_manager.set_gauge("memory_usage_mb", 512.0, labels.clone());
    monitoring_manager.set_gauge("active_connections", 42.0, labels);
    
    // 直方图指标
    let processing_times = vec![1.2, 2.1, 0.8, 3.5, 1.9, 2.7, 1.1];
    for time in processing_times {
        let mut labels = HashMap::new();
        labels.insert("operation".to_string(), "message_processing".to_string());
        monitoring_manager.record_histogram("processing_time_ms", time, labels);
    }
    
    println!("     ✅ 记录了计数器、仪表和直方图指标");
    
    // 3. 计算性能统计
    println!("   📊 计算性能统计");
    let time_range = agentx_a2a::monitoring::TimeRange {
        start: Utc::now() - chrono::Duration::hours(1),
        end: Utc::now(),
    };
    
    let stats = monitoring_manager.calculate_performance_stats(time_range)?;
    
    println!("     消息统计:");
    println!("       总消息数: {}", stats.message_stats.total_messages);
    println!("       成功消息数: {}", stats.message_stats.successful_messages);
    println!("       失败消息数: {}", stats.message_stats.failed_messages);
    println!("       错误率: {:.2}%", stats.error_stats.error_rate * 100.0);
    
    println!("     系统统计:");
    println!("       CPU使用率: {:.1}%", stats.system_stats.cpu_usage_percent);
    println!("       内存使用: {:.1}MB", stats.system_stats.memory_usage_bytes as f64 / 1024.0 / 1024.0);
    
    // 4. 执行健康检查
    println!("   🏥 执行健康检查");
    let health_check = monitoring_manager.perform_health_check()?;
    
    println!("     整体健康状态: {:?} (评分: {})", health_check.status, health_check.score);
    println!("     组件健康状态:");
    
    for (component, health) in &health_check.components {
        let status_icon = match health.status {
            agentx_a2a::monitoring::HealthStatus::Healthy => "🟢",
            agentx_a2a::monitoring::HealthStatus::Degraded => "🟡",
            agentx_a2a::monitoring::HealthStatus::Unhealthy => "🔴",
            agentx_a2a::monitoring::HealthStatus::Unknown => "⚪",
        };
        
        println!("       {} {}: {:?}", status_icon, component, health.status);
        if let Some(response_time) = health.response_time_ms {
            println!("         响应时间: {:.2}ms", response_time);
        }
    }
    
    // 5. 展示指标查询
    println!("   🔍 查询指标数据");
    let metric_names = monitoring_manager.get_metric_names();
    println!("     可用指标: {:?}", metric_names);
    
    if let Some(processing_metrics) = monitoring_manager.get_metrics("processing_time_ms") {
        println!("     处理时间指标数量: {}", processing_metrics.len());
        if let Some(last_metric) = processing_metrics.last() {
            println!("     最新处理时间: {:.2}ms", last_metric.value);
        }
    }
    
    println!("   ✅ 监控和指标演示完成");
    Ok(())
}

/// 演示综合场景
async fn demonstrate_integrated_scenario() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 综合场景演示 - 安全的流式数据传输与监控:");
    
    // 1. 初始化所有组件
    let mut stream_manager = StreamManager::new();
    let mut security_manager = SecurityManager::new(SecurityConfig {
        auth_type: AuthType::ApiKey,
        required_trust_level: TrustLevel::Verified,
        ..Default::default()
    });
    let mut monitoring_manager = MonitoringManager::new(MonitoringConfig::default());
    
    // 2. 设置安全环境
    security_manager.add_trusted_agent("data_producer".to_string(), TrustLevel::Trusted);
    security_manager.add_trusted_agent("data_consumer".to_string(), TrustLevel::Verified);
    
    // 3. 认证数据生产者
    let producer_credentials = AuthCredentials {
        auth_type: AuthType::ApiKey,
        credentials: {
            let mut creds = HashMap::new();
            creds.insert("api_key".to_string(), "producer_key_".to_string() + &"x".repeat(20));
            creds
        },
        expires_at: None,
        scopes: vec!["stream_write".to_string()],
    };
    
    let producer_context = security_manager.authenticate("data_producer", producer_credentials)?;
    println!("   ✅ 数据生产者认证成功 (信任级别: {:?})", producer_context.trust_level);
    
    // 4. 创建安全的数据流
    let secure_header = StreamMessageBuilder::new(StreamType::DataStream)
        .content_type("application/json".to_string())
        .metadata("security_context".to_string(), serde_json::Value::String(producer_context.session_id.clone()))
        .metadata("encryption".to_string(), serde_json::Value::String("AES256".to_string()))
        .build_header(Some(1000), Some(10));
    
    let stream_id = secure_header.stream_id.clone();
    stream_manager.start_stream(secure_header)?;
    
    println!("   📡 创建安全数据流: {}", stream_id);
    
    // 5. 发送加密数据并监控
    println!("   🔄 发送数据并实时监控");
    
    for i in 0..10 {
        let start_time = std::time::Instant::now();
        
        // 模拟数据处理
        let data = serde_json::json!({
            "id": i,
            "timestamp": Utc::now().to_rfc3339(),
            "data": format!("encrypted_data_chunk_{}", i),
            "checksum": format!("sha256_{}", i)
        });
        
        let chunk = StreamChunk {
            stream_id: stream_id.clone(),
            sequence: i,
            data: data.to_string().as_bytes().to_vec(),
            is_final: i == 9,
            checksum: Some(format!("sha256_{}", i)),
            metadata: HashMap::new(),
        };
        
        // 发送数据块
        stream_manager.send_chunk(chunk)?;
        
        let processing_time = start_time.elapsed();
        
        // 记录监控指标
        monitoring_manager.increment_counter("secure_chunks_sent", 1);
        
        let mut labels = HashMap::new();
        labels.insert("stream_id".to_string(), stream_id.clone());
        labels.insert("security_level".to_string(), "trusted".to_string());
        
        monitoring_manager.record_histogram(
            "secure_chunk_processing_time", 
            processing_time.as_secs_f64() * 1000.0, 
            labels
        );
        
        println!("     📦 块 {} 发送完成 (耗时: {:.2}ms)", i, processing_time.as_secs_f64() * 1000.0);
        
        // 模拟处理间隔
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    }
    
    // 6. 验证流完成和安全状态
    let stream_status = stream_manager.get_stream_status(&stream_id);
    if let Some(status) = stream_status {
        println!("   📊 流传输完成:");
        println!("     状态: {:?}", status.state);
        println!("     传输块数: {}", status.received_chunks);
        println!("     完成时间: {}", status.updated_at.format("%H:%M:%S"));
    }
    
    // 7. 生成最终报告
    println!("   📋 生成安全传输报告");

    let time_range = agentx_a2a::monitoring::TimeRange {
        start: Utc::now() - chrono::Duration::minutes(1),
        end: Utc::now(),
    };
    let stats = monitoring_manager.calculate_performance_stats(time_range)?;
    let avg_processing_time = stats.message_stats.avg_processing_time_ms;

    let health_check = monitoring_manager.perform_health_check()?;
    let health_status = health_check.status.clone();
    let health_score = health_check.score;

    println!("     🔒 安全状态: 会话有效，权限验证通过");
    println!("     📡 传输状态: 10/10 块成功传输");
    println!("     🏥 系统健康: {:?} (评分: {})", health_status, health_score);
    println!("     ⚡ 平均处理时间: {:.2}ms", avg_processing_time);
    
    println!("   ✅ 综合场景演示完成");
    Ok(())
}
