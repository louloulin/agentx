//! A2A Protocol Engine Actor
//! 
//! This actor handles A2A protocol message processing with high concurrency
//! and fault tolerance using the Actix actor model.

use actix::prelude::*;
use crate::{
    A2AMessage, A2AError, A2AResult
};
use crate::protocol::ProtocolConfig;
use crate::protocol_engine::A2AProtocolEngine;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// A2A Protocol Engine Actor
pub struct A2AProtocolActor {
    /// A2A protocol engine
    engine: A2AProtocolEngine,

    /// Message processing statistics
    stats: ProtocolStats,

    /// Message processing pool
    handler_pool: Vec<Addr<MessageHandlerActor>>,
}

/// Protocol processing statistics
#[derive(Debug, Clone, Default)]
pub struct ProtocolStats {
    pub messages_processed: u64,
    pub messages_failed: u64,
    pub average_processing_time_ms: f64,
    pub active_handlers: usize,
}

/// Message to process an A2A message
#[derive(Message, Debug, Clone, Serialize, Deserialize)]
#[rtype(result = "A2AResult<Option<A2AMessage>>")]
pub struct ProcessA2AMessage {
    pub message: A2AMessage,
    pub context: ProcessingContext,
}

/// Message processing context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingContext {
    pub source_addr: Option<String>,
    pub processing_id: String,
    pub priority: u8,
    pub timeout_ms: u64,
    pub metadata: HashMap<String, String>,
}

/// Message to register a message handler
#[derive(Message, Debug)]
#[rtype(result = "A2AResult<()>")]
pub struct RegisterHandler {
    pub handler_name: String,
    pub handler: Addr<MessageHandlerActor>,
}

/// Message to get protocol statistics
#[derive(Message, Debug)]
#[rtype(result = "ProtocolStats")]
pub struct GetProtocolStats;

/// Message to update protocol configuration
#[derive(Message, Debug)]
#[rtype(result = "A2AResult<()>")]
pub struct UpdateProtocolConfig {
    pub config: ProtocolConfig,
}

impl Actor for A2AProtocolActor {
    type Context = Context<Self>;
    
    fn started(&mut self, ctx: &mut Self::Context) {
        info!("A2A Protocol Actor started");
        
        // Initialize message handler pool
        self.initialize_handler_pool(ctx);
        
        // Start periodic statistics reporting
        self.start_stats_reporting(ctx);
    }
    
    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("A2A Protocol Actor stopped");
    }
}

impl A2AProtocolActor {
    /// Create a new A2A Protocol Actor
    pub fn new(config: ProtocolConfig) -> Self {
        Self {
            config,
            stats: ProtocolStats::default(),
            handlers: HashMap::new(),
            handler_pool: Vec::new(),
        }
    }
    
    /// Initialize the message handler pool
    fn initialize_handler_pool(&mut self, ctx: &mut Context<Self>) {
        let pool_size = self.config.handler_pool_size.unwrap_or(10);
        
        for i in 0..pool_size {
            let handler = MessageHandlerActor::new(format!("handler-{}", i)).start();
            self.handler_pool.push(handler);
        }
        
        self.stats.active_handlers = pool_size;
        info!("Initialized message handler pool with {} handlers", pool_size);
    }
    
    /// Start periodic statistics reporting
    fn start_stats_reporting(&self, ctx: &mut Context<Self>) {
        ctx.run_interval(
            std::time::Duration::from_secs(30),
            |actor, _ctx| {
                info!("Protocol Stats: {:?}", actor.stats);
            }
        );
    }
    
    /// Select a handler from the pool using round-robin
    fn select_handler(&self) -> Option<&Addr<MessageHandlerActor>> {
        if self.handler_pool.is_empty() {
            return None;
        }
        
        let index = (self.stats.messages_processed as usize) % self.handler_pool.len();
        self.handler_pool.get(index)
    }
    
    /// Validate A2A message
    fn validate_message(&self, message: &A2AMessage) -> A2AResult<()> {
        if !self.config.validate_messages {
            return Ok(());
        }
        
        // Check required fields
        if message.id.is_empty() {
            return Err(A2AError::validation("Message ID is required"));
        }
        
        if message.from.is_empty() {
            return Err(A2AError::validation("Source agent ID is required"));
        }
        
        if message.to.is_empty() {
            return Err(A2AError::validation("Target agent ID is required"));
        }
        
        // Check message size
        let message_size = serde_json::to_string(message)?.len();
        if message_size > self.config.max_message_size {
            return Err(A2AError::validation(
                format!("Message size {} exceeds maximum {}", 
                       message_size, self.config.max_message_size)
            ));
        }
        
        // Check version compatibility
        if message.version != crate::A2A_VERSION {
            return Err(A2AError::version_mismatch(crate::A2A_VERSION, &message.version));
        }
        
        // Check expiration
        if message.is_expired() {
            return Err(A2AError::MessageExpired);
        }
        
        Ok(())
    }
    
    /// Update processing statistics
    fn update_stats(&mut self, processing_time_ms: u64, success: bool) {
        if success {
            self.stats.messages_processed += 1;
        } else {
            self.stats.messages_failed += 1;
        }
        
        // Update average processing time (simple moving average)
        let total_messages = self.stats.messages_processed + self.stats.messages_failed;
        if total_messages > 0 {
            self.stats.average_processing_time_ms = 
                (self.stats.average_processing_time_ms * (total_messages - 1) as f64 + processing_time_ms as f64) 
                / total_messages as f64;
        }
    }
}

/// Handle ProcessA2AMessage
impl Handler<ProcessA2AMessage> for A2AProtocolActor {
    type Result = ResponseActFuture<Self, A2AResult<Option<A2AMessage>>>;
    
    fn handle(&mut self, msg: ProcessA2AMessage, _ctx: &mut Self::Context) -> Self::Result {
        let start_time = std::time::Instant::now();
        
        debug!("Processing A2A message: {} from {} to {}", 
               msg.message.id, msg.message.from, msg.message.to);
        
        // Validate message
        if let Err(e) = self.validate_message(&msg.message) {
            warn!("Message validation failed: {}", e);
            self.update_stats(start_time.elapsed().as_millis() as u64, false);
            return Box::pin(async move { Err(e) }.into_actor(self));
        }
        
        // Select handler
        let handler = match self.select_handler() {
            Some(h) => h.clone(),
            None => {
                error!("No message handlers available");
                self.update_stats(start_time.elapsed().as_millis() as u64, false);
                return Box::pin(async move { 
                    Err(A2AError::internal("No message handlers available")) 
                }.into_actor(self));
            }
        };
        
        // Process message with selected handler
        Box::pin(
            handler
                .send(HandleMessage {
                    message: msg.message,
                    context: msg.context,
                })
                .into_actor(self)
                .map(move |result, actor, _ctx| {
                    let processing_time = start_time.elapsed().as_millis() as u64;
                    
                    match result {
                        Ok(Ok(response)) => {
                            actor.update_stats(processing_time, true);
                            debug!("Message processed successfully in {}ms", processing_time);
                            Ok(response)
                        }
                        Ok(Err(e)) => {
                            actor.update_stats(processing_time, false);
                            error!("Message processing failed: {}", e);
                            Err(e)
                        }
                        Err(e) => {
                            actor.update_stats(processing_time, false);
                            error!("Handler communication failed: {}", e);
                            Err(A2AError::internal(format!("Handler error: {}", e)))
                        }
                    }
                })
        )
    }
}

/// Handle RegisterHandler
impl Handler<RegisterHandler> for A2AProtocolActor {
    type Result = A2AResult<()>;
    
    fn handle(&mut self, msg: RegisterHandler, _ctx: &mut Self::Context) -> Self::Result {
        info!("Registering handler for message type: {:?}", msg.message_type);
        self.handlers.insert(msg.message_type, msg.handler);
        Ok(())
    }
}

/// Handle GetProtocolStats
impl Handler<GetProtocolStats> for A2AProtocolActor {
    type Result = ProtocolStats;
    
    fn handle(&mut self, _msg: GetProtocolStats, _ctx: &mut Self::Context) -> Self::Result {
        self.stats.clone()
    }
}

/// Handle UpdateProtocolConfig
impl Handler<UpdateProtocolConfig> for A2AProtocolActor {
    type Result = A2AResult<()>;
    
    fn handle(&mut self, msg: UpdateProtocolConfig, _ctx: &mut Self::Context) -> Self::Result {
        info!("Updating protocol configuration");
        self.config = msg.config;
        Ok(())
    }
}

/// Message Handler Actor for processing individual messages
pub struct MessageHandlerActor {
    name: String,
    processed_count: u64,
}

impl Actor for MessageHandlerActor {
    type Context = Context<Self>;
}

impl MessageHandlerActor {
    pub fn new(name: String) -> Self {
        Self {
            name,
            processed_count: 0,
        }
    }
}

/// Message to handle an A2A message
#[derive(Message, Debug)]
#[rtype(result = "A2AResult<Option<A2AMessage>>")]
pub struct HandleMessage {
    pub message: A2AMessage,
    pub context: ProcessingContext,
}

/// Handle HandleMessage
impl Handler<HandleMessage> for MessageHandlerActor {
    type Result = A2AResult<Option<A2AMessage>>;
    
    fn handle(&mut self, msg: HandleMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.processed_count += 1;
        
        debug!("Handler {} processing message {}", self.name, msg.message.id);
        
        // Default echo behavior - in real implementation, this would route to appropriate handlers
        match msg.message.message_type {
            MessageType::Request => {
                // Create echo response
                let response = A2AMessage::response(
                    &msg.message,
                    msg.message.payload.clone(),
                );
                Ok(Some(response))
            }
            MessageType::Notification => {
                // Notifications don't require responses
                Ok(None)
            }
            _ => {
                // For other message types, create a simple acknowledgment
                let response = A2AMessage::response(
                    &msg.message,
                    MessagePayload::Text(crate::TextPayload {
                        content: "Message received".to_string(),
                        format: "plain".to_string(),
                        language: None,
                    }),
                );
                Ok(Some(response))
            }
        }
    }
}

impl ProcessingContext {
    pub fn new() -> Self {
        Self {
            source_addr: None,
            processing_id: Uuid::new_v4().to_string(),
            priority: 5,
            timeout_ms: 30000,
            metadata: HashMap::new(),
        }
    }
    
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }
    
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }
}
