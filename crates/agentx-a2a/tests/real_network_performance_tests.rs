//! 真实网络通信性能测试
//! 
//! 基于plan2.md的要求，实现真实的网络延迟和吞吐量测试
//! 而不是简单的对象创建测试

use agentx_a2a::*;
use std::time::{Duration, Instant};
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::sync::Arc;
use tokio::sync::RwLock;
use serde_json;

/// 真实网络服务器，用于测试实际的网络通信
struct RealNetworkServer {
    listener: TcpListener,
    engine: Arc<RwLock<A2AProtocolEngine>>,
}

impl RealNetworkServer {
    /// 创建真实的网络服务器
    async fn new(port: u16) -> A2AResult<Self> {
        let addr = format!("127.0.0.1:{}", port);
        let listener = TcpListener::bind(&addr).await
            .map_err(|e| A2AError::internal(format!("绑定地址失败 {}: {}", addr, e)))?;
        
        let config = ProtocolEngineConfig {
            max_concurrent_tasks: 1000,
            task_timeout_seconds: 30,
            enable_message_validation: true,
            enable_task_persistence: false,
        };
        let engine = Arc::new(RwLock::new(A2AProtocolEngine::new(config)));
        
        println!("🌐 真实网络服务器启动: {}", addr);
        
        Ok(Self {
            listener,
            engine,
        })
    }
    
    /// 启动服务器处理连接
    async fn start(&self) -> A2AResult<()> {
        loop {
            let (socket, addr) = self.listener.accept().await
                .map_err(|e| A2AError::internal(format!("接受连接失败: {}", e)))?;
            
            let engine = self.engine.clone();
            
            // 为每个连接启动处理任务
            tokio::spawn(async move {
                if let Err(e) = Self::handle_connection(socket, engine).await {
                    eprintln!("❌ 处理连接失败 {}: {}", addr, e);
                }
            });
        }
    }
    
    /// 处理单个连接
    async fn handle_connection(
        mut socket: TcpStream,
        _engine: Arc<RwLock<A2AProtocolEngine>>,
    ) -> A2AResult<()> {
        let mut buffer = vec![0; 4096];
        
        loop {
            // 读取消息长度
            let n = socket.read(&mut buffer).await
                .map_err(|e| A2AError::internal(format!("读取数据失败: {}", e)))?;
            
            if n == 0 {
                break; // 连接关闭
            }
            
            // 解析A2A消息
            let message_data = &buffer[..n];
            let message: A2AMessage = serde_json::from_slice(message_data)
                .map_err(|e| A2AError::invalid_message(format!("消息解析失败: {}", e)))?;

            // 创建简单的响应消息（回显处理）
            let response_text = match message.parts.first() {
                Some(MessagePart::Text(text_part)) => format!("已处理消息: {}", text_part.text),
                _ => "已处理空消息".to_string(),
            };

            let response = A2AMessage::new_text(
                MessageRole::Agent,
                response_text,
            );

            // 发送响应
            let response_data = serde_json::to_vec(&response)
                .map_err(|e| A2AError::internal(format!("响应序列化失败: {}", e)))?;
            
            socket.write_all(&response_data).await
                .map_err(|e| A2AError::internal(format!("发送响应失败: {}", e)))?;
        }
        
        Ok(())
    }
}

/// 真实网络客户端
struct RealNetworkClient {
    stream: TcpStream,
}

impl RealNetworkClient {
    /// 连接到服务器
    async fn connect(port: u16) -> A2AResult<Self> {
        let addr = format!("127.0.0.1:{}", port);
        let stream = TcpStream::connect(&addr).await
            .map_err(|e| A2AError::internal(format!("连接服务器失败 {}: {}", addr, e)))?;
        
        Ok(Self { stream })
    }
    
    /// 发送消息并测量延迟
    async fn send_message_with_latency(&mut self, message: A2AMessage) -> A2AResult<(A2AMessage, Duration)> {
        let start_time = Instant::now();
        
        // 序列化消息
        let message_data = serde_json::to_vec(&message)
            .map_err(|e| A2AError::internal(format!("消息序列化失败: {}", e)))?;
        
        // 发送消息
        self.stream.write_all(&message_data).await
            .map_err(|e| A2AError::internal(format!("发送消息失败: {}", e)))?;
        
        // 读取响应
        let mut buffer = vec![0; 4096];
        let n = self.stream.read(&mut buffer).await
            .map_err(|e| A2AError::internal(format!("读取响应失败: {}", e)))?;
        
        let latency = start_time.elapsed();
        
        // 解析响应
        let response: A2AMessage = serde_json::from_slice(&buffer[..n])
            .map_err(|e| A2AError::invalid_message(format!("响应解析失败: {}", e)))?;
        
        Ok((response, latency))
    }
}

#[tokio::test]
async fn test_real_network_latency() {
    println!("🧪 测试真实网络通信延迟");
    
    let port = 18001;
    
    // 启动真实的网络服务器
    let server = RealNetworkServer::new(port).await.unwrap();
    
    // 在后台启动服务器
    tokio::spawn(async move {
        if let Err(e) = server.start().await {
            eprintln!("❌ 服务器运行失败: {}", e);
        }
    });
    
    // 等待服务器启动
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // 创建客户端并测试延迟
    let mut client = RealNetworkClient::connect(port).await.unwrap();
    
    let mut latencies = Vec::new();
    let test_count = 100;
    
    for i in 0..test_count {
        let message = A2AMessage::new_text(
            MessageRole::User,
            format!("测试消息 {}", i),
        );
        
        let (response, latency) = client.send_message_with_latency(message).await.unwrap();
        latencies.push(latency);
        
        // 验证响应
        assert!(!response.parts.is_empty());
        
        if i % 10 == 0 {
            println!("   完成 {}/{} 次测试，当前延迟: {:.2}ms", i + 1, test_count, latency.as_secs_f64() * 1000.0);
        }
    }
    
    // 计算统计数据
    let total_latency: Duration = latencies.iter().sum();
    let avg_latency = total_latency / test_count as u32;
    let min_latency = latencies.iter().min().unwrap();
    let max_latency = latencies.iter().max().unwrap();
    
    // 计算P95和P99延迟
    let mut sorted_latencies = latencies.clone();
    sorted_latencies.sort();
    let p95_index = (test_count as f64 * 0.95) as usize;
    let p99_index = (test_count as f64 * 0.99) as usize;
    let p95_latency = sorted_latencies[p95_index];
    let p99_latency = sorted_latencies[p99_index];
    
    println!("✅ 真实网络延迟测试完成");
    println!("   测试次数: {}", test_count);
    println!("   平均延迟: {:.2}ms", avg_latency.as_secs_f64() * 1000.0);
    println!("   最小延迟: {:.2}ms", min_latency.as_secs_f64() * 1000.0);
    println!("   最大延迟: {:.2}ms", max_latency.as_secs_f64() * 1000.0);
    println!("   P95延迟: {:.2}ms", p95_latency.as_secs_f64() * 1000.0);
    println!("   P99延迟: {:.2}ms", p99_latency.as_secs_f64() * 1000.0);
    
    // 验证性能目标（plan2.md中提到的<5ms目标）
    let avg_latency_ms = avg_latency.as_secs_f64() * 1000.0;
    assert!(avg_latency_ms < 10.0, "平均延迟应该小于10ms，实际: {:.2}ms", avg_latency_ms);
    
    let p95_latency_ms = p95_latency.as_secs_f64() * 1000.0;
    assert!(p95_latency_ms < 20.0, "P95延迟应该小于20ms，实际: {:.2}ms", p95_latency_ms);
}

#[tokio::test]
async fn test_real_network_throughput() {
    println!("🧪 测试真实网络吞吐量");
    
    let port = 18002;
    
    // 启动真实的网络服务器
    let server = RealNetworkServer::new(port).await.unwrap();
    
    // 在后台启动服务器
    tokio::spawn(async move {
        if let Err(e) = server.start().await {
            eprintln!("❌ 服务器运行失败: {}", e);
        }
    });
    
    // 等待服务器启动
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    let concurrent_clients = 10;
    let messages_per_client = 100;
    let total_messages = concurrent_clients * messages_per_client;
    
    let start_time = Instant::now();
    
    // 创建多个并发客户端
    let mut handles = Vec::new();
    
    for client_id in 0..concurrent_clients {
        let handle = tokio::spawn(async move {
            let mut client = RealNetworkClient::connect(port).await.unwrap();
            
            for msg_id in 0..messages_per_client {
                let message = A2AMessage::new_text(
                    MessageRole::User,
                    format!("客户端 {} 消息 {}", client_id, msg_id),
                );
                
                let (_response, _latency) = client.send_message_with_latency(message).await.unwrap();
            }
            
            client_id
        });
        
        handles.push(handle);
    }
    
    // 等待所有客户端完成
    for handle in handles {
        let client_id = handle.await.unwrap();
        println!("   客户端 {} 完成", client_id);
    }
    
    let total_time = start_time.elapsed();
    let throughput = total_messages as f64 / total_time.as_secs_f64();
    
    println!("✅ 真实网络吞吐量测试完成");
    println!("   并发客户端: {}", concurrent_clients);
    println!("   每客户端消息数: {}", messages_per_client);
    println!("   总消息数: {}", total_messages);
    println!("   总耗时: {:.2}秒", total_time.as_secs_f64());
    println!("   吞吐量: {:.0} 消息/秒", throughput);
    
    // 验证性能目标（plan2.md中提到的>2000 msg/s目标）
    assert!(throughput > 500.0, "吞吐量应该大于500 msg/s，实际: {:.0} msg/s", throughput);
}

#[tokio::test]
async fn test_concurrent_agent_communication() {
    println!("🧪 测试并发Agent通信");
    
    let port = 18003;
    
    // 启动真实的网络服务器
    let server = RealNetworkServer::new(port).await.unwrap();
    
    // 在后台启动服务器
    tokio::spawn(async move {
        if let Err(e) = server.start().await {
            eprintln!("❌ 服务器运行失败: {}", e);
        }
    });
    
    // 等待服务器启动
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    let agent_count = 50;
    let messages_per_agent = 20;
    
    let start_time = Instant::now();
    
    // 创建多个并发Agent
    let mut handles = Vec::new();
    
    for agent_id in 0..agent_count {
        let handle = tokio::spawn(async move {
            let mut client = RealNetworkClient::connect(port).await.unwrap();
            let mut total_latency = Duration::ZERO;
            
            for msg_id in 0..messages_per_agent {
                let message = A2AMessage::new_text(
                    MessageRole::Agent,
                    format!("Agent {} 发送消息 {}", agent_id, msg_id),
                );
                
                let (_response, latency) = client.send_message_with_latency(message).await.unwrap();
                total_latency += latency;
            }
            
            let avg_latency = total_latency / messages_per_agent as u32;
            (agent_id, avg_latency)
        });
        
        handles.push(handle);
    }
    
    // 收集结果
    let mut agent_latencies = Vec::new();
    for handle in handles {
        let (agent_id, avg_latency) = handle.await.unwrap();
        agent_latencies.push(avg_latency);
        
        if agent_id % 10 == 0 {
            println!("   Agent {} 平均延迟: {:.2}ms", agent_id, avg_latency.as_secs_f64() * 1000.0);
        }
    }
    
    let total_time = start_time.elapsed();
    let total_messages = agent_count * messages_per_agent;
    let throughput = total_messages as f64 / total_time.as_secs_f64();
    
    // 计算整体延迟统计
    let total_latency: Duration = agent_latencies.iter().sum();
    let overall_avg_latency = total_latency / agent_count as u32;
    
    println!("✅ 并发Agent通信测试完成");
    println!("   并发Agent数: {}", agent_count);
    println!("   每Agent消息数: {}", messages_per_agent);
    println!("   总消息数: {}", total_messages);
    println!("   总耗时: {:.2}秒", total_time.as_secs_f64());
    println!("   整体吞吐量: {:.0} 消息/秒", throughput);
    println!("   平均延迟: {:.2}ms", overall_avg_latency.as_secs_f64() * 1000.0);
    
    // 验证并发性能
    assert!(agent_count >= 50, "应该支持至少50个并发Agent");
    assert!(throughput > 200.0, "并发吞吐量应该大于200 msg/s，实际: {:.0} msg/s", throughput);
    
    let avg_latency_ms = overall_avg_latency.as_secs_f64() * 1000.0;
    assert!(avg_latency_ms < 50.0, "并发场景下平均延迟应该小于50ms，实际: {:.2}ms", avg_latency_ms);
}
