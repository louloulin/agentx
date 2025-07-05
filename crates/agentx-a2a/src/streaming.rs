//! A2A协议流式通信支持
//! 
//! 实现A2A协议的流式消息传输，支持大文件传输、实时数据流和长时间运行的任务

use crate::{A2AMessage, A2AError, A2AResult, MessageRole};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::mpsc;
use uuid::Uuid;

/// 流式消息类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StreamType {
    /// 数据流 - 连续的数据传输
    DataStream,
    /// 文件流 - 大文件分块传输
    FileStream,
    /// 事件流 - 实时事件推送
    EventStream,
    /// 任务流 - 长时间运行任务的进度更新
    TaskStream,
    /// 音频流 - 实时音频数据
    AudioStream,
    /// 视频流 - 实时视频数据
    VideoStream,
}

/// 流式消息状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StreamState {
    /// 流开始
    Started,
    /// 流进行中
    Streaming,
    /// 流暂停
    Paused,
    /// 流恢复
    Resumed,
    /// 流完成
    Completed,
    /// 流错误
    Error(String),
    /// 流取消
    Cancelled,
}

/// 流式消息块
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChunk {
    /// 流ID
    pub stream_id: String,
    /// 块序号
    pub sequence: u64,
    /// 块数据
    pub data: Vec<u8>,
    /// 是否为最后一块
    pub is_final: bool,
    /// 块校验和
    pub checksum: Option<String>,
    /// 元数据
    pub metadata: HashMap<String, serde_json::Value>,
}

/// 流式消息头
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamHeader {
    /// 流ID
    pub stream_id: String,
    /// 流类型
    pub stream_type: StreamType,
    /// 流状态
    pub state: StreamState,
    /// 总大小（字节）
    pub total_size: Option<u64>,
    /// 预期块数量
    pub expected_chunks: Option<u64>,
    /// 内容类型
    pub content_type: Option<String>,
    /// 编码方式
    pub encoding: Option<String>,
    /// 元数据
    pub metadata: HashMap<String, serde_json::Value>,
}

/// 流式消息管理器
#[derive(Debug)]
pub struct StreamManager {
    /// 活跃的流
    active_streams: HashMap<String, StreamInfo>,
    /// 消息发送器
    message_sender: mpsc::UnboundedSender<A2AMessage>,
    /// 消息接收器
    #[allow(dead_code)]
    message_receiver: Option<mpsc::UnboundedReceiver<A2AMessage>>,
}

/// 流信息
#[derive(Debug)]
struct StreamInfo {
    /// 流头信息
    header: StreamHeader,
    /// 已接收的块
    received_chunks: HashMap<u64, StreamChunk>,
    /// 下一个期望的序号
    next_sequence: u64,
    /// 流状态
    state: StreamState,
    /// 创建时间
    created_at: chrono::DateTime<chrono::Utc>,
    /// 最后更新时间
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl StreamManager {
    /// 创建新的流管理器
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        
        Self {
            active_streams: HashMap::new(),
            message_sender: sender,
            message_receiver: Some(receiver),
        }
    }
    
    /// 开始新的流
    pub fn start_stream(&mut self, header: StreamHeader) -> A2AResult<()> {
        let stream_id = header.stream_id.clone();
        
        if self.active_streams.contains_key(&stream_id) {
            return Err(A2AError::InvalidMessage(
                format!("流 {} 已存在", stream_id)
            ));
        }
        
        let now = chrono::Utc::now();
        let stream_info = StreamInfo {
            header: header.clone(),
            received_chunks: HashMap::new(),
            next_sequence: 0,
            state: StreamState::Started,
            created_at: now,
            updated_at: now,
        };
        
        self.active_streams.insert(stream_id.clone(), stream_info);
        
        // 发送流开始消息
        let start_message = A2AMessage::new_data(
            MessageRole::Agent,
            serde_json::json!({
                "type": "stream_start",
                "stream_header": header
            }),
        );
        
        self.message_sender.send(start_message)
            .map_err(|_| A2AError::internal("无法发送流开始消息"))?;
        
        Ok(())
    }
    
    /// 发送流数据块
    pub fn send_chunk(&mut self, chunk: StreamChunk) -> A2AResult<()> {
        let stream_id = chunk.stream_id.clone();
        
        let stream_info = self.active_streams.get_mut(&stream_id)
            .ok_or_else(|| A2AError::InvalidMessage(
                format!("流 {} 不存在", stream_id)
            ))?;
        
        // 验证序号
        if chunk.sequence != stream_info.next_sequence {
            return Err(A2AError::InvalidMessage(
                format!("期望序号 {}，但收到 {}", stream_info.next_sequence, chunk.sequence)
            ));
        }
        
        // 更新流信息
        stream_info.received_chunks.insert(chunk.sequence, chunk.clone());
        stream_info.next_sequence += 1;
        stream_info.updated_at = chrono::Utc::now();
        
        // 发送块消息
        let chunk_message = A2AMessage::new_data(
            MessageRole::Agent,
            serde_json::json!({
                "type": "stream_chunk",
                "chunk": chunk
            }),
        );
        
        self.message_sender.send(chunk_message)
            .map_err(|_| A2AError::internal("无法发送流数据块"))?;
        
        // 如果是最后一块，完成流
        if chunk.is_final {
            self.complete_stream(&stream_id)?;
        }
        
        Ok(())
    }
    
    /// 完成流
    pub fn complete_stream(&mut self, stream_id: &str) -> A2AResult<()> {
        let stream_info = self.active_streams.get_mut(stream_id)
            .ok_or_else(|| A2AError::InvalidMessage(
                format!("流 {} 不存在", stream_id)
            ))?;
        
        stream_info.state = StreamState::Completed;
        stream_info.updated_at = chrono::Utc::now();
        
        // 发送流完成消息
        let complete_message = A2AMessage::new_data(
            MessageRole::Agent,
            serde_json::json!({
                "type": "stream_complete",
                "stream_id": stream_id,
                "total_chunks": stream_info.received_chunks.len(),
                "completed_at": stream_info.updated_at
            }),
        );
        
        self.message_sender.send(complete_message)
            .map_err(|_| A2AError::internal("无法发送流完成消息"))?;
        
        Ok(())
    }
    
    /// 取消流
    pub fn cancel_stream(&mut self, stream_id: &str, reason: Option<String>) -> A2AResult<()> {
        let stream_info = self.active_streams.get_mut(stream_id)
            .ok_or_else(|| A2AError::InvalidMessage(
                format!("流 {} 不存在", stream_id)
            ))?;
        
        stream_info.state = StreamState::Cancelled;
        stream_info.updated_at = chrono::Utc::now();
        
        // 发送流取消消息
        let cancel_message = A2AMessage::new_data(
            MessageRole::Agent,
            serde_json::json!({
                "type": "stream_cancel",
                "stream_id": stream_id,
                "reason": reason,
                "cancelled_at": stream_info.updated_at
            }),
        );
        
        self.message_sender.send(cancel_message)
            .map_err(|_| A2AError::internal("无法发送流取消消息"))?;
        
        Ok(())
    }
    
    /// 获取流状态
    pub fn get_stream_status(&self, stream_id: &str) -> Option<StreamStatus> {
        self.active_streams.get(stream_id).map(|info| {
            StreamStatus {
                stream_id: stream_id.to_string(),
                state: info.state.clone(),
                total_chunks: info.header.expected_chunks,
                received_chunks: info.received_chunks.len() as u64,
                progress: if let Some(total) = info.header.expected_chunks {
                    Some(info.received_chunks.len() as f64 / total as f64)
                } else {
                    None
                },
                created_at: info.created_at,
                updated_at: info.updated_at,
            }
        })
    }
    
    /// 获取所有活跃流的状态
    pub fn get_all_streams(&self) -> Vec<StreamStatus> {
        self.active_streams.iter().map(|(stream_id, info)| {
            StreamStatus {
                stream_id: stream_id.clone(),
                state: info.state.clone(),
                total_chunks: info.header.expected_chunks,
                received_chunks: info.received_chunks.len() as u64,
                progress: if let Some(total) = info.header.expected_chunks {
                    Some(info.received_chunks.len() as f64 / total as f64)
                } else {
                    None
                },
                created_at: info.created_at,
                updated_at: info.updated_at,
            }
        }).collect()
    }
    
    /// 清理已完成的流
    pub fn cleanup_completed_streams(&mut self) {
        let now = chrono::Utc::now();
        let cleanup_threshold = chrono::Duration::hours(1);
        
        self.active_streams.retain(|_, info| {
            match info.state {
                StreamState::Completed | StreamState::Cancelled | StreamState::Error(_) => {
                    now.signed_duration_since(info.updated_at) < cleanup_threshold
                },
                _ => true,
            }
        });
    }
}

/// 流状态信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamStatus {
    /// 流ID
    pub stream_id: String,
    /// 流状态
    pub state: StreamState,
    /// 总块数
    pub total_chunks: Option<u64>,
    /// 已接收块数
    pub received_chunks: u64,
    /// 进度百分比
    pub progress: Option<f64>,
    /// 创建时间
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// 更新时间
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// 流式消息构建器
pub struct StreamMessageBuilder {
    stream_type: StreamType,
    content_type: Option<String>,
    encoding: Option<String>,
    metadata: HashMap<String, serde_json::Value>,
}

impl StreamMessageBuilder {
    /// 创建新的流消息构建器
    pub fn new(stream_type: StreamType) -> Self {
        Self {
            stream_type,
            content_type: None,
            encoding: None,
            metadata: HashMap::new(),
        }
    }
    
    /// 设置内容类型
    pub fn content_type(mut self, content_type: String) -> Self {
        self.content_type = Some(content_type);
        self
    }
    
    /// 设置编码方式
    pub fn encoding(mut self, encoding: String) -> Self {
        self.encoding = Some(encoding);
        self
    }
    
    /// 添加元数据
    pub fn metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }
    
    /// 构建流头
    pub fn build_header(self, total_size: Option<u64>, expected_chunks: Option<u64>) -> StreamHeader {
        StreamHeader {
            stream_id: Uuid::new_v4().to_string(),
            stream_type: self.stream_type,
            state: StreamState::Started,
            total_size,
            expected_chunks,
            content_type: self.content_type,
            encoding: self.encoding,
            metadata: self.metadata,
        }
    }
}

impl Default for StreamManager {
    fn default() -> Self {
        Self::new()
    }
}
