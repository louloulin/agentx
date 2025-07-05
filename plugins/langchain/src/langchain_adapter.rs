//! LangChain框架适配器
//! 
//! 提供LangChain框架与A2A协议的适配功能

use crate::python_bridge::PythonBridge;
use crate::error::{LangChainError, LangChainResult};
use agentx_grpc::proto::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{info, debug, error, warn};
use serde_json::{json, Value};

/// LangChain适配器
pub struct LangChainAdapter {
    /// Python桥接器
    python_bridge: Arc<PythonBridge>,
    /// Agent实例缓存
    agents: Arc<RwLock<HashMap<String, LangChainAgent>>>,
    /// 会话管理器
    sessions: Arc<RwLock<HashMap<String, ConversationSession>>>,
}

/// LangChain Agent实例
#[derive(Debug, Clone)]
pub struct LangChainAgent {
    pub id: String,
    pub name: String,
    pub agent_type: String,
    pub model_name: String,
    pub tools: Vec<String>,
    pub memory_type: Option<String>,
    pub config: HashMap<String, Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// 对话会话
#[derive(Debug, Clone)]
pub struct ConversationSession {
    pub session_id: String,
    pub agent_id: String,
    pub messages: Vec<SessionMessage>,
    pub context: HashMap<String, Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// 会话消息
#[derive(Debug, Clone)]
pub struct SessionMessage {
    pub role: String,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub metadata: HashMap<String, Value>,
}

impl LangChainAdapter {
    /// 创建新的LangChain适配器
    pub async fn new(python_bridge: Arc<PythonBridge>) -> LangChainResult<Self> {
        info!("🔧 初始化LangChain适配器");
        
        Ok(Self {
            python_bridge,
            agents: Arc::new(RwLock::new(HashMap::new())),
            sessions: Arc::new(RwLock::new(HashMap::new())),
        })
    }
    
    /// 初始化适配器
    pub async fn initialize(&self) -> LangChainResult<()> {
        info!("🚀 初始化LangChain适配器...");
        
        // 初始化Python环境
        self.python_bridge.initialize_langchain_environment().await?;
        
        // 预加载常用模型和工具
        self.preload_common_components().await?;
        
        info!("✅ LangChain适配器初始化完成");
        Ok(())
    }
    
    /// 关闭适配器
    pub async fn shutdown(&self) -> LangChainResult<()> {
        info!("🛑 关闭LangChain适配器...");
        
        // 清理所有会话
        {
            let mut sessions = self.sessions.write().await;
            sessions.clear();
        }
        
        // 清理所有Agent
        {
            let mut agents = self.agents.write().await;
            agents.clear();
        }
        
        info!("✅ LangChain适配器已关闭");
        Ok(())
    }
    
    /// 处理A2A消息
    pub async fn process_message(&self, request: A2aMessageRequest) -> LangChainResult<A2aMessageRequest> {
        debug!("📨 处理A2A消息: {}", request.message_id);
        
        // 解析消息类型和内容
        let message_type = MessageType::from_i32(request.message_type).unwrap_or(MessageType::MessageTypeUnspecified);
        
        match message_type {
            MessageType::MessageTypeRequest => {
                self.handle_request_message(request).await
            }
            MessageType::MessageTypeEvent => {
                self.handle_event_message(request).await
            }
            MessageType::MessageTypeStream => {
                self.handle_stream_message(request).await
            }
            _ => {
                warn!("不支持的消息类型: {:?}", message_type);
                Err(LangChainError::UnsupportedMessageType(format!("{:?}", message_type)))
            }
        }
    }
    
    /// 处理请求消息
    async fn handle_request_message(&self, request: A2aMessageRequest) -> LangChainResult<A2aMessageRequest> {
        debug!("🔄 处理请求消息");
        
        // 提取消息内容
        let payload = request.payload.as_ref()
            .ok_or_else(|| LangChainError::InvalidMessage("缺少消息载荷".to_string()))?;
        
        // 解析为JSON
        let content: Value = serde_json::from_slice(&payload.value)
            .map_err(|e| LangChainError::InvalidMessage(format!("JSON解析失败: {}", e)))?;
        
        // 确定目标Agent
        let agent_id = request.to_agent.clone();
        let agent = self.get_agent(&agent_id).await?;
        
        // 获取或创建会话
        let session_id = request.metadata.get("session_id")
            .cloned()
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        
        let mut session = self.get_or_create_session(&session_id, &agent_id).await?;
        
        // 处理不同类型的请求
        let response_content = match content.get("type").and_then(|t| t.as_str()) {
            Some("chat") => {
                let user_message = content.get("message")
                    .and_then(|m| m.as_str())
                    .ok_or_else(|| LangChainError::InvalidMessage("缺少聊天消息".to_string()))?;
                
                self.handle_chat_request(&agent, &mut session, user_message).await?
            }
            Some("tool_call") => {
                let tool_name = content.get("tool_name")
                    .and_then(|t| t.as_str())
                    .ok_or_else(|| LangChainError::InvalidMessage("缺少工具名称".to_string()))?;
                
                let tool_args = content.get("arguments")
                    .cloned()
                    .unwrap_or(json!({}));
                
                self.handle_tool_call(&agent, tool_name, &tool_args).await?
            }
            Some("chain_execution") => {
                let chain_config = content.get("chain")
                    .ok_or_else(|| LangChainError::InvalidMessage("缺少链配置".to_string()))?;
                
                self.handle_chain_execution(&agent, chain_config).await?
            }
            _ => {
                return Err(LangChainError::UnsupportedOperation(
                    "不支持的请求类型".to_string()
                ));
            }
        };
        
        // 更新会话
        self.update_session(session).await?;
        
        // 构建响应消息
        let response_payload = json!({
            "type": "response",
            "content": response_content,
            "agent_id": agent_id,
            "session_id": session_id,
            "timestamp": chrono::Utc::now().to_rfc3339()
        });
        
        Ok(A2aMessageRequest {
            message_id: uuid::Uuid::new_v4().to_string(),
            from_agent: agent_id,
            to_agent: request.from_agent,
            message_type: MessageType::MessageTypeResponse as i32,
            payload: Some(prost_types::Any {
                type_url: "type.googleapis.com/agentx.a2a.TextPayload".to_string(),
                value: serde_json::to_vec(&response_payload)
                    .map_err(|e| LangChainError::SerializationError(e.to_string()))?,
            }),
            metadata: HashMap::new(),
            timestamp: Some(prost_types::Timestamp::from(std::time::SystemTime::now())),
            ttl_seconds: 300,
        })
    }
    
    /// 处理聊天请求
    async fn handle_chat_request(
        &self,
        agent: &LangChainAgent,
        session: &mut ConversationSession,
        user_message: &str,
    ) -> LangChainResult<Value> {
        debug!("💬 处理聊天请求: {}", user_message);
        
        // 添加用户消息到会话
        session.messages.push(SessionMessage {
            role: "user".to_string(),
            content: user_message.to_string(),
            timestamp: chrono::Utc::now(),
            metadata: HashMap::new(),
        });
        
        // 构建聊天请求
        let chat_request = json!({
            "agent_type": agent.agent_type,
            "model": agent.model_name,
            "messages": session.messages.iter().map(|m| json!({
                "role": m.role,
                "content": m.content
            })).collect::<Vec<_>>(),
            "tools": agent.tools,
            "memory_type": agent.memory_type,
            "config": agent.config
        });
        
        // 调用Python LangChain
        let response = self.python_bridge.call_langchain_chat(chat_request).await?;
        
        // 解析响应
        let assistant_message = response.get("content")
            .and_then(|c| c.as_str())
            .ok_or_else(|| LangChainError::InvalidResponse("缺少响应内容".to_string()))?;
        
        // 添加助手消息到会话
        session.messages.push(SessionMessage {
            role: "assistant".to_string(),
            content: assistant_message.to_string(),
            timestamp: chrono::Utc::now(),
            metadata: HashMap::new(),
        });
        
        session.updated_at = chrono::Utc::now();
        
        Ok(json!({
            "message": assistant_message,
            "usage": response.get("usage"),
            "model": agent.model_name
        }))
    }
    
    /// 处理工具调用
    async fn handle_tool_call(
        &self,
        agent: &LangChainAgent,
        tool_name: &str,
        tool_args: &Value,
    ) -> LangChainResult<Value> {
        debug!("🔧 处理工具调用: {}", tool_name);
        
        // 验证工具是否可用
        if !agent.tools.contains(&tool_name.to_string()) {
            return Err(LangChainError::ToolNotFound(tool_name.to_string()));
        }
        
        // 构建工具调用请求
        let tool_request = json!({
            "tool_name": tool_name,
            "arguments": tool_args,
            "agent_config": agent.config
        });
        
        // 调用Python LangChain工具
        let response = self.python_bridge.call_langchain_tool(tool_request).await?;
        
        Ok(response)
    }
    
    /// 处理链执行
    async fn handle_chain_execution(
        &self,
        agent: &LangChainAgent,
        chain_config: &Value,
    ) -> LangChainResult<Value> {
        debug!("⛓️ 处理链执行");
        
        // 构建链执行请求
        let chain_request = json!({
            "chain_config": chain_config,
            "agent_config": agent.config,
            "model": agent.model_name
        });
        
        // 调用Python LangChain链
        let response = self.python_bridge.call_langchain_chain(chain_request).await?;
        
        Ok(response)
    }
    
    /// 处理事件消息
    async fn handle_event_message(&self, request: A2aMessageRequest) -> LangChainResult<A2aMessageRequest> {
        debug!("📢 处理事件消息");
        
        // TODO: 实现事件消息处理
        Ok(request)
    }
    
    /// 处理流式消息
    async fn handle_stream_message(&self, request: A2aMessageRequest) -> LangChainResult<A2aMessageRequest> {
        debug!("🌊 处理流式消息");
        
        // TODO: 实现流式消息处理
        Ok(request)
    }
    
    /// 处理流式数据
    pub async fn process_stream(
        &self,
        mut stream: tonic::Streaming<A2aStreamChunk>,
        tx: mpsc::Sender<Result<A2aStreamChunk, tonic::Status>>,
    ) -> LangChainResult<()> {
        info!("🌊 开始处理流式数据");
        
        while let Some(chunk) = stream.message().await.map_err(|e| {
            LangChainError::StreamError(format!("流读取错误: {}", e))
        })? {
            debug!("📦 处理流块: {}", chunk.stream_id);
            
            // 处理流块
            let processed_chunk = self.process_stream_chunk(chunk).await?;
            
            // 发送处理后的块
            if tx.send(Ok(processed_chunk)).await.is_err() {
                warn!("流接收器已关闭");
                break;
            }
        }
        
        info!("✅ 流式数据处理完成");
        Ok(())
    }
    
    /// 处理单个流块
    async fn process_stream_chunk(&self, chunk: A2aStreamChunk) -> LangChainResult<A2aStreamChunk> {
        // TODO: 实现流块处理逻辑
        Ok(chunk)
    }
    
    /// 注册Agent
    pub async fn register_agent(
        &self,
        agent_info: AgentInfo,
        capabilities: Vec<Capability>,
    ) -> LangChainResult<String> {
        info!("📝 注册LangChain Agent: {}", agent_info.name);
        
        // 创建LangChain Agent实例
        let agent = LangChainAgent {
            id: agent_info.id.clone(),
            name: agent_info.name.clone(),
            agent_type: agent_info.metadata.get("agent_type")
                .cloned()
                .unwrap_or_else(|| "conversational".to_string()),
            model_name: agent_info.metadata.get("model")
                .cloned()
                .unwrap_or_else(|| "gpt-3.5-turbo".to_string()),
            tools: capabilities.iter()
                .filter(|c| c.r#type == CapabilityType::CapabilityTypeTool as i32)
                .map(|c| c.name.clone())
                .collect(),
            memory_type: agent_info.metadata.get("memory_type").cloned(),
            config: agent_info.metadata.iter()
                .map(|(k, v)| (k.clone(), json!(v)))
                .collect(),
            created_at: chrono::Utc::now(),
        };
        
        // 在Python环境中创建Agent
        let create_request = json!({
            "agent_id": agent.id,
            "agent_type": agent.agent_type,
            "model": agent.model_name,
            "tools": agent.tools,
            "memory_type": agent.memory_type,
            "config": agent.config
        });
        
        self.python_bridge.create_langchain_agent(create_request).await?;
        
        // 缓存Agent
        {
            let mut agents = self.agents.write().await;
            agents.insert(agent.id.clone(), agent);
        }
        
        info!("✅ LangChain Agent注册成功: {}", agent_info.id);
        Ok(agent_info.id)
    }
    
    /// 注销Agent
    pub async fn unregister_agent(&self, agent_id: &str) -> LangChainResult<()> {
        info!("🗑️ 注销LangChain Agent: {}", agent_id);
        
        // 从缓存中移除
        {
            let mut agents = self.agents.write().await;
            agents.remove(agent_id);
        }
        
        // 清理相关会话
        {
            let mut sessions = self.sessions.write().await;
            sessions.retain(|_, session| session.agent_id != agent_id);
        }
        
        // 在Python环境中删除Agent
        self.python_bridge.delete_langchain_agent(agent_id).await?;
        
        info!("✅ LangChain Agent注销完成: {}", agent_id);
        Ok(())
    }
    
    /// 获取Agent能力
    pub async fn get_agent_capabilities(&self, agent_id: &str) -> LangChainResult<Vec<Capability>> {
        let agent = self.get_agent(agent_id).await?;
        
        let mut capabilities = vec![
            Capability {
                id: "chat".to_string(),
                name: "聊天对话".to_string(),
                description: "与用户进行自然语言对话".to_string(),
                r#type: CapabilityType::CapabilityTypeSkill as i32,
                parameters: vec![],
                returns: vec![],
                metadata: HashMap::new(),
            }
        ];
        
        // 添加工具能力
        for tool in &agent.tools {
            capabilities.push(Capability {
                id: tool.clone(),
                name: tool.clone(),
                description: format!("LangChain工具: {}", tool),
                r#type: CapabilityType::CapabilityTypeTool as i32,
                parameters: vec![],
                returns: vec![],
                metadata: HashMap::new(),
            });
        }
        
        Ok(capabilities)
    }
    
    /// 获取Agent
    async fn get_agent(&self, agent_id: &str) -> LangChainResult<LangChainAgent> {
        let agents = self.agents.read().await;
        agents.get(agent_id)
            .cloned()
            .ok_or_else(|| LangChainError::AgentNotFound(agent_id.to_string()))
    }
    
    /// 获取或创建会话
    async fn get_or_create_session(
        &self,
        session_id: &str,
        agent_id: &str,
    ) -> LangChainResult<ConversationSession> {
        let sessions = self.sessions.read().await;
        
        if let Some(session) = sessions.get(session_id) {
            Ok(session.clone())
        } else {
            drop(sessions);
            
            let session = ConversationSession {
                session_id: session_id.to_string(),
                agent_id: agent_id.to_string(),
                messages: Vec::new(),
                context: HashMap::new(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            };
            
            let mut sessions = self.sessions.write().await;
            sessions.insert(session_id.to_string(), session.clone());
            
            Ok(session)
        }
    }
    
    /// 更新会话
    async fn update_session(&self, session: ConversationSession) -> LangChainResult<()> {
        let mut sessions = self.sessions.write().await;
        sessions.insert(session.session_id.clone(), session);
        Ok(())
    }
    
    /// 预加载常用组件
    async fn preload_common_components(&self) -> LangChainResult<()> {
        info!("📦 预加载LangChain常用组件...");
        
        // 预加载常用模型
        let models = vec!["gpt-3.5-turbo", "gpt-4", "claude-3-sonnet"];
        for model in models {
            if let Err(e) = self.python_bridge.preload_model(model).await {
                warn!("预加载模型 {} 失败: {}", model, e);
            }
        }
        
        // 预加载常用工具
        let tools = vec!["search", "calculator", "weather", "web_scraper"];
        for tool in tools {
            if let Err(e) = self.python_bridge.preload_tool(tool).await {
                warn!("预加载工具 {} 失败: {}", tool, e);
            }
        }
        
        info!("✅ 常用组件预加载完成");
        Ok(())
    }
}
