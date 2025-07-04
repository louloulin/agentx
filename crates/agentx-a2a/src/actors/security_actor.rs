//! Security Manager Actor
//! 
//! This actor handles authentication, authorization, and security auditing
//! for the A2A protocol using the Actix actor model.

use actix::prelude::*;
use crate::{A2AError, A2AResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// Security Manager Actor
pub struct SecurityManagerActor {
    /// Active sessions
    sessions: HashMap<String, Session>,
    
    /// Security policies
    policies: HashMap<String, SecurityPolicy>,
    
    /// Audit log
    audit_log: Vec<AuditEvent>,
    
    /// Security configuration
    config: SecurityConfig,
}

/// User session information
#[derive(Debug, Clone)]
pub struct Session {
    pub session_id: String,
    pub user_id: String,
    pub agent_id: Option<String>,
    pub permissions: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_activity: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

/// Security policy
#[derive(Debug, Clone)]
pub struct SecurityPolicy {
    pub name: String,
    pub rules: Vec<SecurityRule>,
    pub enabled: bool,
}

/// Security rule
#[derive(Debug, Clone)]
pub struct SecurityRule {
    pub resource: String,
    pub action: String,
    pub effect: RuleEffect,
    pub conditions: Vec<String>,
}

/// Rule effect
#[derive(Debug, Clone, PartialEq)]
pub enum RuleEffect {
    Allow,
    Deny,
}

/// Audit event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub event_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub event_type: AuditEventType,
    pub user_id: Option<String>,
    pub agent_id: Option<String>,
    pub resource: String,
    pub action: String,
    pub result: AuditResult,
    pub details: HashMap<String, String>,
}

/// Audit event type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditEventType {
    Authentication,
    Authorization,
    MessageProcessing,
    AgentRegistration,
    ConfigurationChange,
}

/// Audit result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditResult {
    Success,
    Failure,
    Warning,
}

/// Security configuration
#[derive(Debug, Clone)]
pub struct SecurityConfig {
    pub session_timeout_minutes: u64,
    pub max_sessions_per_user: usize,
    pub audit_log_max_size: usize,
    pub enable_audit_logging: bool,
}

/// Message to authenticate a user
#[derive(Message, Debug, Clone, Serialize, Deserialize)]
#[rtype(result = "A2AResult<AuthenticationResult>")]
pub struct Authenticate {
    pub credentials: Credentials,
    pub agent_id: Option<String>,
}

/// Authentication credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    pub auth_type: AuthenticationType,
    pub token: String,
    pub metadata: HashMap<String, String>,
}

/// Authentication type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthenticationType {
    Bearer,
    ApiKey,
    OAuth2,
    Certificate,
}

/// Authentication result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationResult {
    pub success: bool,
    pub session_id: Option<String>,
    pub user_id: Option<String>,
    pub permissions: Vec<String>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub error: Option<String>,
}

/// Message to authorize an action
#[derive(Message, Debug, Clone, Serialize, Deserialize)]
#[rtype(result = "A2AResult<AuthorizationResult>")]
pub struct Authorize {
    pub session_id: String,
    pub resource: String,
    pub action: String,
    pub context: HashMap<String, String>,
}

/// Authorization result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationResult {
    pub allowed: bool,
    pub reason: Option<String>,
}

/// Message to invalidate a session
#[derive(Message, Debug, Clone, Serialize, Deserialize)]
#[rtype(result = "A2AResult<()>")]
pub struct InvalidateSession {
    pub session_id: String,
}

/// Message to get audit events
#[derive(Message, Debug, Clone, Serialize, Deserialize)]
#[rtype(result = "Vec<AuditEvent>")]
pub struct GetAuditEvents {
    pub filter: Option<AuditFilter>,
    pub limit: Option<usize>,
}

/// Audit filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditFilter {
    pub event_type: Option<AuditEventType>,
    pub user_id: Option<String>,
    pub agent_id: Option<String>,
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
}

impl Actor for SecurityManagerActor {
    type Context = Context<Self>;
    
    fn started(&mut self, ctx: &mut Self::Context) {
        info!("Security Manager Actor started");
        
        // Start session cleanup task
        self.start_session_cleanup(ctx);
        
        // Start audit log cleanup task
        self.start_audit_cleanup(ctx);
    }
    
    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("Security Manager Actor stopped");
    }
}

impl SecurityManagerActor {
    /// Create a new Security Manager Actor
    pub fn new(config: SecurityConfig) -> Self {
        Self {
            sessions: HashMap::new(),
            policies: HashMap::new(),
            audit_log: Vec::new(),
            config,
        }
    }
    
    /// Start session cleanup task
    fn start_session_cleanup(&self, ctx: &mut Context<Self>) {
        ctx.run_interval(
            std::time::Duration::from_secs(300), // 5 minutes
            |actor, _ctx| {
                actor.cleanup_expired_sessions();
            }
        );
    }
    
    /// Start audit log cleanup task
    fn start_audit_cleanup(&self, ctx: &mut Context<Self>) {
        ctx.run_interval(
            std::time::Duration::from_secs(3600), // 1 hour
            |actor, _ctx| {
                actor.cleanup_audit_log();
            }
        );
    }
    
    /// Cleanup expired sessions
    fn cleanup_expired_sessions(&mut self) {
        let now = chrono::Utc::now();
        let expired_sessions: Vec<String> = self.sessions
            .iter()
            .filter(|(_, session)| session.expires_at < now)
            .map(|(id, _)| id.clone())
            .collect();
        
        for session_id in expired_sessions {
            self.sessions.remove(&session_id);
            debug!("Removed expired session: {}", session_id);
        }
    }
    
    /// Cleanup audit log
    fn cleanup_audit_log(&mut self) {
        if self.audit_log.len() > self.config.audit_log_max_size {
            let excess = self.audit_log.len() - self.config.audit_log_max_size;
            self.audit_log.drain(0..excess);
            debug!("Cleaned up {} old audit events", excess);
        }
    }
    
    /// Validate credentials
    fn validate_credentials(&self, credentials: &Credentials) -> A2AResult<(String, Vec<String>)> {
        match credentials.auth_type {
            AuthenticationType::Bearer => {
                // Simple bearer token validation
                if credentials.token.starts_with("bearer_") {
                    let user_id = credentials.token.replace("bearer_", "");
                    let permissions = vec!["read".to_string(), "write".to_string()];
                    Ok((user_id, permissions))
                } else {
                    Err(A2AError::authentication("Invalid bearer token"))
                }
            }
            AuthenticationType::ApiKey => {
                // Simple API key validation
                if credentials.token.starts_with("api_") {
                    let user_id = credentials.token.replace("api_", "");
                    let permissions = vec!["read".to_string()];
                    Ok((user_id, permissions))
                } else {
                    Err(A2AError::authentication("Invalid API key"))
                }
            }
            _ => Err(A2AError::authentication("Unsupported authentication type")),
        }
    }
    
    /// Create a new session
    fn create_session(&mut self, user_id: String, agent_id: Option<String>, permissions: Vec<String>) -> String {
        let session_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now();
        let expires_at = now + chrono::Duration::minutes(self.config.session_timeout_minutes as i64);
        
        let session = Session {
            session_id: session_id.clone(),
            user_id: user_id.clone(),
            agent_id,
            permissions,
            created_at: now,
            last_activity: now,
            expires_at,
        };
        
        self.sessions.insert(session_id.clone(), session);
        
        // Log authentication event
        self.log_audit_event(AuditEvent {
            event_id: uuid::Uuid::new_v4().to_string(),
            timestamp: now,
            event_type: AuditEventType::Authentication,
            user_id: Some(user_id),
            agent_id: None,
            resource: "session".to_string(),
            action: "create".to_string(),
            result: AuditResult::Success,
            details: HashMap::new(),
        });
        
        session_id
    }
    
    /// Check authorization
    fn check_authorization(&self, session_id: &str, resource: &str, action: &str) -> AuthorizationResult {
        if let Some(session) = self.sessions.get(session_id) {
            // Simple permission check
            let allowed = session.permissions.contains(&action.to_string()) ||
                         session.permissions.contains(&"admin".to_string());
            
            AuthorizationResult {
                allowed,
                reason: if allowed { None } else { Some("Insufficient permissions".to_string()) },
            }
        } else {
            AuthorizationResult {
                allowed: false,
                reason: Some("Invalid session".to_string()),
            }
        }
    }
    
    /// Log audit event
    fn log_audit_event(&mut self, event: AuditEvent) {
        if self.config.enable_audit_logging {
            self.audit_log.push(event);
        }
    }
}

/// Handle Authenticate
impl Handler<Authenticate> for SecurityManagerActor {
    type Result = A2AResult<AuthenticationResult>;
    
    fn handle(&mut self, msg: Authenticate, _ctx: &mut Self::Context) -> Self::Result {
        debug!("Authenticating user with {:?} credentials", msg.credentials.auth_type);
        
        match self.validate_credentials(&msg.credentials) {
            Ok((user_id, permissions)) => {
                let session_id = self.create_session(user_id.clone(), msg.agent_id, permissions.clone());
                let expires_at = self.sessions.get(&session_id).map(|s| s.expires_at);
                
                Ok(AuthenticationResult {
                    success: true,
                    session_id: Some(session_id),
                    user_id: Some(user_id),
                    permissions,
                    expires_at,
                    error: None,
                })
            }
            Err(e) => {
                warn!("Authentication failed: {}", e);
                
                // Log failed authentication
                self.log_audit_event(AuditEvent {
                    event_id: uuid::Uuid::new_v4().to_string(),
                    timestamp: chrono::Utc::now(),
                    event_type: AuditEventType::Authentication,
                    user_id: None,
                    agent_id: msg.agent_id,
                    resource: "session".to_string(),
                    action: "create".to_string(),
                    result: AuditResult::Failure,
                    details: [("error".to_string(), e.to_string())].iter().cloned().collect(),
                });
                
                Ok(AuthenticationResult {
                    success: false,
                    session_id: None,
                    user_id: None,
                    permissions: Vec::new(),
                    expires_at: None,
                    error: Some(e.to_string()),
                })
            }
        }
    }
}

/// Handle Authorize
impl Handler<Authorize> for SecurityManagerActor {
    type Result = A2AResult<AuthorizationResult>;
    
    fn handle(&mut self, msg: Authorize, _ctx: &mut Self::Context) -> Self::Result {
        debug!("Authorizing action {} on resource {} for session {}", 
               msg.action, msg.resource, msg.session_id);
        
        let result = self.check_authorization(&msg.session_id, &msg.resource, &msg.action);
        
        // Update session activity
        if let Some(session) = self.sessions.get_mut(&msg.session_id) {
            session.last_activity = chrono::Utc::now();
        }
        
        // Log authorization event
        self.log_audit_event(AuditEvent {
            event_id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            event_type: AuditEventType::Authorization,
            user_id: self.sessions.get(&msg.session_id).map(|s| s.user_id.clone()),
            agent_id: self.sessions.get(&msg.session_id).and_then(|s| s.agent_id.clone()),
            resource: msg.resource,
            action: msg.action,
            result: if result.allowed { AuditResult::Success } else { AuditResult::Failure },
            details: HashMap::new(),
        });
        
        Ok(result)
    }
}

/// Handle InvalidateSession
impl Handler<InvalidateSession> for SecurityManagerActor {
    type Result = A2AResult<()>;
    
    fn handle(&mut self, msg: InvalidateSession, _ctx: &mut Self::Context) -> Self::Result {
        debug!("Invalidating session: {}", msg.session_id);
        
        if let Some(session) = self.sessions.remove(&msg.session_id) {
            // Log session invalidation
            self.log_audit_event(AuditEvent {
                event_id: uuid::Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now(),
                event_type: AuditEventType::Authentication,
                user_id: Some(session.user_id),
                agent_id: session.agent_id,
                resource: "session".to_string(),
                action: "invalidate".to_string(),
                result: AuditResult::Success,
                details: HashMap::new(),
            });
        }
        
        Ok(())
    }
}

/// Handle GetAuditEvents
impl Handler<GetAuditEvents> for SecurityManagerActor {
    type Result = Vec<AuditEvent>;
    
    fn handle(&mut self, msg: GetAuditEvents, _ctx: &mut Self::Context) -> Self::Result {
        let mut events = self.audit_log.clone();
        
        // Apply filters
        if let Some(filter) = msg.filter {
            events.retain(|event| {
                if let Some(event_type) = &filter.event_type {
                    if std::mem::discriminant(&event.event_type) != std::mem::discriminant(event_type) {
                        return false;
                    }
                }
                
                if let Some(user_id) = &filter.user_id {
                    if event.user_id.as_ref() != Some(user_id) {
                        return false;
                    }
                }
                
                if let Some(agent_id) = &filter.agent_id {
                    if event.agent_id.as_ref() != Some(agent_id) {
                        return false;
                    }
                }
                
                if let Some(start_time) = filter.start_time {
                    if event.timestamp < start_time {
                        return false;
                    }
                }
                
                if let Some(end_time) = filter.end_time {
                    if event.timestamp > end_time {
                        return false;
                    }
                }
                
                true
            });
        }
        
        // Apply limit
        if let Some(limit) = msg.limit {
            events.truncate(limit);
        }
        
        events
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            session_timeout_minutes: 60,
            max_sessions_per_user: 5,
            audit_log_max_size: 10000,
            enable_audit_logging: true,
        }
    }
}
