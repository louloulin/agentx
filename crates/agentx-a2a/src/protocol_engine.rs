//! A2A Protocol Engine Implementation
//! 
//! This module implements the core A2A protocol engine that handles
//! message routing, task management, and agent communication according
//! to the A2A protocol specification v0.2.5.

use crate::{
    A2AMessage, A2ATask, JsonRpcRequest, JsonRpcResponse, JsonRpcError,
    MessageRole, TaskState, TaskStatus, methods, A2AError, A2AResult
};
use chrono::Utc;
use serde_json::Value;
use std::collections::HashMap;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

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

/// Agent status
#[derive(Debug, Clone, PartialEq)]
pub enum AgentStatus {
    Online,
    Offline,
    Busy,
}

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
        
        // For now, create a simple echo response
        // In a real implementation, this would route to the appropriate agent
        let response = A2AMessage::agent_message(
            format!("Echo: {}", message.get_text_content().unwrap_or_default())
        ).with_task_id(message.task_id.unwrap_or_default())
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
                    state,
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
