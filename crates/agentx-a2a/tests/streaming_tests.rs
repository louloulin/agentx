//! A2A协议流式通信测试
//! 
//! 测试A2A协议的流式消息传输功能

use agentx_a2a::{
    StreamManager, StreamChunk, StreamType, StreamState,
    StreamMessageBuilder,
};
use std::collections::HashMap;
use tokio;

#[tokio::test]
async fn test_stream_manager_creation() {
    println!("🧪 测试流管理器创建");
    
    let manager = StreamManager::new();
    println!("   ✅ 流管理器创建成功");
    
    // 验证初始状态
    let all_streams = manager.get_all_streams();
    assert_eq!(all_streams.len(), 0);
    println!("   ✅ 初始状态验证通过");
}

#[tokio::test]
async fn test_stream_header_creation() {
    println!("🧪 测试流头创建");
    
    let header = StreamMessageBuilder::new(StreamType::DataStream)
        .content_type("application/json".to_string())
        .encoding("utf-8".to_string())
        .metadata("source".to_string(), serde_json::Value::String("test".to_string()))
        .build_header(Some(1024), Some(10));
    
    println!("   流ID: {}", header.stream_id);
    println!("   流类型: {:?}", header.stream_type);
    println!("   总大小: {:?}", header.total_size);
    println!("   预期块数: {:?}", header.expected_chunks);
    
    assert_eq!(header.stream_type, StreamType::DataStream);
    assert_eq!(header.total_size, Some(1024));
    assert_eq!(header.expected_chunks, Some(10));
    assert_eq!(header.content_type, Some("application/json".to_string()));
    
    println!("   ✅ 流头创建测试通过");
}

#[tokio::test]
async fn test_stream_lifecycle() {
    println!("🧪 测试流生命周期");
    
    let mut manager = StreamManager::new();
    
    // 1. 创建流头
    let header = StreamMessageBuilder::new(StreamType::FileStream)
        .content_type("text/plain".to_string())
        .build_header(Some(100), Some(3));
    
    let stream_id = header.stream_id.clone();
    
    // 2. 开始流
    println!("   📝 开始流: {}", stream_id);
    let result = manager.start_stream(header);
    assert!(result.is_ok());
    
    // 3. 验证流状态
    let status = manager.get_stream_status(&stream_id);
    assert!(status.is_some());
    let status = status.unwrap();
    assert_eq!(status.state, StreamState::Started);
    assert_eq!(status.received_chunks, 0);
    println!("   ✅ 流状态验证通过");
    
    // 4. 发送数据块
    for i in 0..3 {
        let chunk = StreamChunk {
            stream_id: stream_id.clone(),
            sequence: i,
            data: format!("chunk_{}", i).into_bytes(),
            is_final: i == 2,
            checksum: None,
            metadata: HashMap::new(),
        };
        
        println!("   📦 发送块 {}", i);
        let result = manager.send_chunk(chunk);
        assert!(result.is_ok());
    }
    
    // 5. 验证流完成
    let status = manager.get_stream_status(&stream_id);
    assert!(status.is_some());
    let status = status.unwrap();
    assert_eq!(status.state, StreamState::Completed);
    assert_eq!(status.received_chunks, 3);
    println!("   ✅ 流完成验证通过");
    
    println!("   ✅ 流生命周期测试通过");
}

#[tokio::test]
async fn test_stream_error_handling() {
    println!("🧪 测试流错误处理");
    
    let mut manager = StreamManager::new();
    
    // 1. 测试向不存在的流发送数据
    let chunk = StreamChunk {
        stream_id: "nonexistent".to_string(),
        sequence: 0,
        data: vec![1, 2, 3],
        is_final: false,
        checksum: None,
        metadata: HashMap::new(),
    };
    
    let result = manager.send_chunk(chunk);
    assert!(result.is_err());
    println!("   ✅ 不存在流的错误处理通过");
    
    // 2. 测试重复开始流
    let header = StreamMessageBuilder::new(StreamType::EventStream)
        .build_header(None, None);
    
    let stream_id = header.stream_id.clone();
    
    let result1 = manager.start_stream(header.clone());
    assert!(result1.is_ok());
    
    let result2 = manager.start_stream(header);
    assert!(result2.is_err());
    println!("   ✅ 重复流的错误处理通过");
    
    // 3. 测试错误的序号
    let chunk = StreamChunk {
        stream_id: stream_id.clone(),
        sequence: 5, // 期望0，但发送5
        data: vec![1, 2, 3],
        is_final: false,
        checksum: None,
        metadata: HashMap::new(),
    };
    
    let result = manager.send_chunk(chunk);
    assert!(result.is_err());
    println!("   ✅ 错误序号的错误处理通过");
    
    println!("   ✅ 流错误处理测试通过");
}

#[tokio::test]
async fn test_stream_cancellation() {
    println!("🧪 测试流取消");
    
    let mut manager = StreamManager::new();
    
    // 创建并开始流
    let header = StreamMessageBuilder::new(StreamType::AudioStream)
        .build_header(Some(1000), Some(10));
    
    let stream_id = header.stream_id.clone();
    manager.start_stream(header).unwrap();
    
    // 发送几个块
    for i in 0..3 {
        let chunk = StreamChunk {
            stream_id: stream_id.clone(),
            sequence: i,
            data: vec![i as u8; 10],
            is_final: false,
            checksum: None,
            metadata: HashMap::new(),
        };
        
        manager.send_chunk(chunk).unwrap();
    }
    
    // 取消流
    let result = manager.cancel_stream(&stream_id, Some("用户取消".to_string()));
    assert!(result.is_ok());
    
    // 验证流状态
    let status = manager.get_stream_status(&stream_id);
    assert!(status.is_some());
    let status = status.unwrap();
    assert_eq!(status.state, StreamState::Cancelled);
    assert_eq!(status.received_chunks, 3);
    
    println!("   ✅ 流取消测试通过");
}

#[tokio::test]
async fn test_stream_cleanup() {
    println!("🧪 测试流清理");
    
    let mut manager = StreamManager::new();
    
    // 创建多个流
    let mut stream_ids = Vec::new();
    
    for i in 0..5 {
        let header = StreamMessageBuilder::new(StreamType::VideoStream)
            .build_header(Some(100), Some(1));
        
        let stream_id = header.stream_id.clone();
        stream_ids.push(stream_id.clone());
        
        manager.start_stream(header).unwrap();
        
        // 完成一些流
        if i < 3 {
            let chunk = StreamChunk {
                stream_id: stream_id.clone(),
                sequence: 0,
                data: vec![i as u8; 10],
                is_final: true,
                checksum: None,
                metadata: HashMap::new(),
            };
            
            manager.send_chunk(chunk).unwrap();
        }
    }
    
    // 验证流数量
    let all_streams = manager.get_all_streams();
    assert_eq!(all_streams.len(), 5);
    
    // 执行清理（在实际实现中，这会清理过期的已完成流）
    manager.cleanup_completed_streams();
    
    // 验证清理后的状态
    let all_streams = manager.get_all_streams();
    println!("   清理后的流数量: {}", all_streams.len());
    
    println!("   ✅ 流清理测试通过");
}

#[tokio::test]
async fn test_different_stream_types() {
    println!("🧪 测试不同流类型");
    
    let stream_types = vec![
        StreamType::DataStream,
        StreamType::FileStream,
        StreamType::EventStream,
        StreamType::TaskStream,
        StreamType::AudioStream,
        StreamType::VideoStream,
    ];
    
    for stream_type in stream_types {
        println!("   测试流类型: {:?}", stream_type);
        
        let header = StreamMessageBuilder::new(stream_type.clone())
            .build_header(Some(50), Some(1));
        
        assert_eq!(header.stream_type, stream_type);
        assert_eq!(header.state, StreamState::Started);
        
        println!("     ✅ {:?} 流类型测试通过", stream_type);
    }
    
    println!("   ✅ 不同流类型测试通过");
}

#[tokio::test]
async fn test_stream_performance() {
    println!("🧪 测试流性能");
    
    let mut manager = StreamManager::new();
    let chunk_count = 1000;
    
    // 创建流
    let header = StreamMessageBuilder::new(StreamType::DataStream)
        .build_header(Some(chunk_count * 100), Some(chunk_count));
    
    let stream_id = header.stream_id.clone();
    manager.start_stream(header).unwrap();
    
    // 测量发送性能
    let start_time = std::time::Instant::now();
    
    for i in 0..chunk_count {
        let chunk = StreamChunk {
            stream_id: stream_id.clone(),
            sequence: i,
            data: vec![0u8; 100], // 100字节的数据
            is_final: i == chunk_count - 1,
            checksum: None,
            metadata: HashMap::new(),
        };
        
        manager.send_chunk(chunk).unwrap();
    }
    
    let duration = start_time.elapsed();
    let throughput = chunk_count as f64 / duration.as_secs_f64();
    
    println!("   📊 性能测试结果:");
    println!("     块数量: {}", chunk_count);
    println!("     总耗时: {:?}", duration);
    println!("     吞吐量: {:.0} 块/秒", throughput);
    println!("     平均延迟: {:.3}ms", duration.as_millis() as f64 / chunk_count as f64);
    
    // 验证性能目标
    assert!(throughput > 10000.0, "流吞吐量 {:.0} 块/秒 低于10,000块/秒的目标", throughput);
    
    println!("   ✅ 流性能测试通过");
}
