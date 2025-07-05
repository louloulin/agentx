//! A2A Protocol Engine Implementation
//! 
//! This module implements the core A2A protocol engine that handles
//! message routing, task management, and agent communication according
//! to the A2A protocol specification v0.2.5.

use crate::{
    A2AMessage, A2ATask, TaskState, TaskStatus, A2AError, A2AResult,
    JsonRpcRequest, JsonRpcResponse, JsonRpcError, methods
};
use crate::agent_card::AgentStatus;
use chrono::Utc;
use std::collections::HashMap;
use tracing::{debug, error, info, warn};
use uuid;

/// A2A Protocol Engine - Core implementation of A2A protocol
pub struct A2AProtocolEngine {
    /// Active tasks managed by this engine
    tasks: HashMap<String, A2ATask>,
    
    /// Agent registry for routing
    agents: HashMap<String, AgentInfo>,
    
    /// Protocol configuration
    config: ProtocolEngineConfig,
    
    /// Engine statistics
    stats: EngineStats,
}

/// Agent information for routing
#[derive(Debug, Clone)]
pub struct AgentInfo {
    pub id: String,
    pub name: String,
    pub endpoint: String,
    pub capabilities: Vec<String>,
    pub status: AgentStatus,
}

// AgentStatus is defined in agent_card.rs

/// Protocol engine configuration
#[derive(Debug, Clone)]
pub struct ProtocolEngineConfig {
    pub max_concurrent_tasks: usize,
    pub task_timeout_seconds: u64,
    pub enable_message_validation: bool,
    pub enable_task_persistence: bool,
}

/// Engine statistics
#[derive(Debug, Clone, Default)]
pub struct EngineStats {
    pub total_tasks: u64,
    pub completed_tasks: u64,
    pub failed_tasks: u64,
    pub active_tasks: usize,
    pub messages_processed: u64,
    pub messages_routed: u64,
}

impl A2AProtocolEngine {
    /// Create a new A2A protocol engine
    pub fn new(config: ProtocolEngineConfig) -> Self {
        Self {
            tasks: HashMap::new(),
            agents: HashMap::new(),
            config,
            stats: EngineStats::default(),
        }
    }
    
    /// Process a JSON-RPC request according to A2A protocol
    pub async fn process_request(&mut self, request: JsonRpcRequest) -> JsonRpcResponse {
        debug!("Processing A2A request: {}", request.method);
        
        match request.method.as_str() {
            methods::SUBMIT_TASK => self.handle_submit_task(request).await,
            methods::GET_TASK => self.handle_get_task(request).await,
            methods::CANCEL_TASK => self.handle_cancel_task(request).await,
            methods::SEND_MESSAGE => self.handle_send_message(request).await,
            methods::GET_CAPABILITIES => self.handle_get_capabilities(request).await,
            _ => JsonRpcResponse::error(
                JsonRpcError::method_not_found(),
                request.id
            ),
        }
    }
    
    /// Handle submitTask request
    async fn handle_submit_task(&mut self, request: JsonRpcRequest) -> JsonRpcResponse {
        let params = match request.params {
            Some(params) => params,
            None => return JsonRpcResponse::error(
                JsonRpcError::invalid_params(),
                request.id
            ),
        };
        
        let task: A2ATask = match serde_json::from_value(params) {
            Ok(task) => task,
            Err(e) => {
                error!("Failed to parse task: {}", e);
                return JsonRpcResponse::error(
                    JsonRpcError::invalid_params(),
                    request.id
                );
            }
        };
        
        // Validate task
        if let Err(e) = self.validate_task(&task) {
            warn!("Task validation failed: {}", e);
            return JsonRpcResponse::error(
                JsonRpcError::new(-32000, e.to_string(), None),
                request.id
            );
        }
        
        // Store task
        let task_id = task.id.clone();
        self.tasks.insert(task_id.clone(), task);
        
        // Update statistics
        self.stats.total_tasks += 1;
        self.stats.active_tasks = self.tasks.len();
        
        info!("Task {} submitted successfully", task_id);
        
        JsonRpcResponse::success(
            serde_json::json!({
                "taskId": task_id,
                "status": "submitted"
            }),
            request.id
        )
    }
    
    /// Handle getTask request
    async fn handle_get_task(&mut self, request: JsonRpcRequest) -> JsonRpcResponse {
        let params = match request.params {
            Some(params) => params,
            None => return JsonRpcResponse::error(
                JsonRpcError::invalid_params(),
                request.id
            ),
        };
        
        let task_id = match params.get("taskId").and_then(|v| v.as_str()) {
            Some(id) => id,
            None => return JsonRpcResponse::error(
                JsonRpcError::invalid_params(),
                request.id
            ),
        };
        
        match self.tasks.get(task_id) {
            Some(task) => {
                debug!("Retrieved task: {}", task_id);
                JsonRpcResponse::success(
                    serde_json::to_value(task).unwrap(),
                    request.id
                )
            }
            None => JsonRpcResponse::error(
                JsonRpcError::new(-32001, "Task not found".to_string(), None),
                request.id
            ),
        }
    }
    
    /// Handle cancelTask request
    async fn handle_cancel_task(&mut self, request: JsonRpcRequest) -> JsonRpcResponse {
        let params = match request.params {
            Some(params) => params,
            None => return JsonRpcResponse::error(
                JsonRpcError::invalid_params(),
                request.id
            ),
        };
        
        let task_id = match params.get("taskId").and_then(|v| v.as_str()) {
            Some(id) => id,
            None => return JsonRpcResponse::error(
                JsonRpcError::invalid_params(),
                request.id
            ),
        };
        
        match self.tasks.get_mut(task_id) {
            Some(task) => {
                task.status = TaskStatus {
                    state: TaskState::Canceled,
                    timestamp: Some(Utc::now()),
                    message: None,
                };
                
                info!("Task {} canceled", task_id);
                
                JsonRpcResponse::success(
                    serde_json::json!({
                        "taskId": task_id,
                        "status": "canceled"
                    }),
                    request.id
                )
            }
            None => JsonRpcResponse::error(
                JsonRpcError::new(-32001, "Task not found".to_string(), None),
                request.id
            ),
        }
    }
    
    /// Handle sendMessage request
    async fn handle_send_message(&mut self, request: JsonRpcRequest) -> JsonRpcResponse {
        let params = match request.params {
            Some(params) => params,
            None => return JsonRpcResponse::error(
                JsonRpcError::invalid_params(),
                request.id
            ),
        };
        
        let message: A2AMessage = match serde_json::from_value(params) {
            Ok(message) => message,
            Err(e) => {
                error!("Failed to parse message: {}", e);
                return JsonRpcResponse::error(
                    JsonRpcError::invalid_params(),
                    request.id
                );
            }
        };
        
        // Route message
        match self.route_message(message).await {
            Ok(response) => {
                self.stats.messages_processed += 1;
                self.stats.messages_routed += 1;
                
                JsonRpcResponse::success(
                    serde_json::json!({
                        "messageId": response.message_id,
                        "status": "delivered"
                    }),
                    request.id
                )
            }
            Err(e) => {
                error!("Failed to route message: {}", e);
                JsonRpcResponse::error(
                    JsonRpcError::new(-32002, e.to_string(), None),
                    request.id
                )
            }
        }
    }
    
    /// Handle getCapabilities request
    async fn handle_get_capabilities(&mut self, request: JsonRpcRequest) -> JsonRpcResponse {
        let capabilities = self.agents.values()
            .flat_map(|agent| agent.capabilities.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        
        JsonRpcResponse::success(
            serde_json::json!({
                "capabilities": capabilities,
                "agents": self.agents.len()
            }),
            request.id
        )
    }
    
    /// Validate a task according to A2A protocol
    fn validate_task(&self, task: &A2ATask) -> A2AResult<()> {
        if !self.config.enable_message_validation {
            return Ok(());
        }
        
        if task.id.is_empty() {
            return Err(A2AError::validation("Task ID cannot be empty"));
        }
        
        if task.kind.is_empty() {
            return Err(A2AError::validation("Task kind cannot be empty"));
        }
        
        if self.tasks.len() >= self.config.max_concurrent_tasks {
            return Err(A2AError::internal("Maximum concurrent tasks reached"));
        }
        
        Ok(())
    }
    
    /// Route a message to the appropriate agent
    async fn route_message(&mut self, message: A2AMessage) -> A2AResult<A2AMessage> {
        debug!("Routing message: {}", message.message_id);

        // 实现真实的消息路由逻辑
        // 1. 检查消息是否有目标任务ID
        if let Some(task_id) = &message.task_id {
            if let Some(task) = self.tasks.get_mut(task_id) {
                // 将消息添加到任务历史
                task.history.push(message.clone());

                // 根据消息角色处理
                match message.role {
                    crate::MessageRole::User => {
                        // 用户消息，更新任务状态为工作中
                        task.status.state = TaskState::Working;
                        task.status.timestamp = Some(chrono::Utc::now());

                        // 创建处理响应
                        let response = A2AMessage::agent_message(
                            format!("Processing your request: {}",
                                message.get_text_content().unwrap_or_default())
                        ).with_task_id(task_id.clone())
                         .with_context_id(message.context_id.unwrap_or_default());

                        return Ok(response);
                    },
                    crate::MessageRole::Agent => {
                        // Agent消息，可能是任务完成
                        task.status.state = TaskState::Completed;
                        task.status.timestamp = Some(chrono::Utc::now());

                        // 创建确认响应
                        let response = A2AMessage::agent_message(
                            "Task completed successfully".to_string()
                        ).with_task_id(task_id.clone())
                         .with_context_id(message.context_id.unwrap_or_default());

                        return Ok(response);
                    }
                }
            }
        }

        // 2. 如果没有任务ID，创建新任务
        let task_id = uuid::Uuid::new_v4().to_string();
        let mut new_task = A2ATask::new("message_processing".to_string());
        new_task.id = task_id.clone();
        new_task.context_id = message.context_id.clone();
        new_task.history.push(message.clone());

        // 添加任务到引擎
        self.tasks.insert(task_id.clone(), new_task);

        // 创建响应消息
        let response = A2AMessage::agent_message(
            format!("Created new task for: {}",
                message.get_text_content().unwrap_or_default())
        ).with_task_id(task_id)
         .with_context_id(message.context_id.unwrap_or_default());

        Ok(response)
    }
    
    /// Register an agent with the engine
    pub fn register_agent(&mut self, agent: AgentInfo) {
        info!("Registering agent: {}", agent.id);
        self.agents.insert(agent.id.clone(), agent);
    }
    
    /// Unregister an agent from the engine
    pub fn unregister_agent(&mut self, agent_id: &str) {
        info!("Unregistering agent: {}", agent_id);
        self.agents.remove(agent_id);
    }

    /// Get list of all registered agents
    pub fn list_agents(&self) -> Vec<AgentInfo> {
        self.agents.values().cloned().collect()
    }

    /// Get agent by ID
    pub fn get_agent(&self, agent_id: &str) -> Option<&AgentInfo> {
        self.agents.get(agent_id)
    }
    
    /// Get engine statistics
    pub fn get_stats(&self) -> &EngineStats {
        &self.stats
    }
    
    /// Get active tasks count
    pub fn get_active_tasks_count(&self) -> usize {
        self.tasks.len()
    }
    
    /// Update task status
    pub fn update_task_status(&mut self, task_id: &str, state: TaskState) -> A2AResult<()> {
        match self.tasks.get_mut(task_id) {
            Some(task) => {
                task.status = TaskStatus {
                    state: state.clone(),
                    timestamp: Some(Utc::now()),
                    message: None,
                };

                // Update statistics
                match state {
                    TaskState::Completed => self.stats.completed_tasks += 1,
                    TaskState::Failed => self.stats.failed_tasks += 1,
                    _ => {}
                }
                
                Ok(())
            }
            None => Err(A2AError::agent_not_found(task_id)),
        }
    }
}

impl Default for ProtocolEngineConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: 1000,
            task_timeout_seconds: 300, // 5 minutes
            enable_message_validation: true,
            enable_task_persistence: false,
        }
    }
}
