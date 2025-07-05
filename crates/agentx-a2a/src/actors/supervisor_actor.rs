//! Plugin Supervisor Actor
//! 
//! This actor supervises gRPC plugin processes and handles their lifecycle,
//! health monitoring, and fault recovery using the Actix actor model.

use actix::prelude::*;
use crate::{A2AError, A2AResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::{Child, Command, Stdio};
use tracing::{debug, error, info, warn};

/// Plugin Supervisor Actor
pub struct PluginSupervisorActor {
    /// Managed plugin processes
    plugins: HashMap<String, PluginProcess>,
    
    /// Supervisor statistics
    stats: SupervisorStats,
    
    /// Supervisor configuration
    config: SupervisorConfig,
}

/// Plugin process information
#[derive(Debug, Clone)]
pub struct PluginProcess {
    pub id: String,
    pub name: String,
    pub executable_path: String,
    pub process: Option<Child>,
    pub grpc_port: u16,
    pub status: PluginStatus,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub restart_count: u32,
    pub last_health_check: Option<chrono::DateTime<chrono::Utc>>,
}

/// Plugin status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PluginStatus {
    Starting,
    Running,
    Stopping,
    Stopped,
    Failed,
    Unknown,
}

/// Supervisor statistics
#[derive(Debug, Clone, Default)]
pub struct SupervisorStats {
    pub total_plugins: usize,
    pub running_plugins: usize,
    pub failed_plugins: usize,
    pub total_restarts: u32,
    pub health_checks_performed: u64,
}

/// Supervisor configuration
#[derive(Debug, Clone)]
pub struct SupervisorConfig {
    pub max_restart_attempts: u32,
    pub restart_delay_ms: u64,
    pub health_check_interval_ms: u64,
    pub startup_timeout_ms: u64,
    pub shutdown_timeout_ms: u64,
}

/// Message to start a plugin
#[derive(Message, Debug, Clone, Serialize, Deserialize)]
#[rtype(result = "A2AResult<()>")]
pub struct StartPlugin {
    pub plugin_id: String,
    pub executable_path: String,
    pub grpc_port: u16,
    pub config: HashMap<String, String>,
}

/// Message to stop a plugin
#[derive(Message, Debug, Clone, Serialize, Deserialize)]
#[rtype(result = "A2AResult<()>")]
pub struct StopPlugin {
    pub plugin_id: String,
    pub force: bool,
}

/// Message to restart a plugin
#[derive(Message, Debug, Clone, Serialize, Deserialize)]
#[rtype(result = "A2AResult<()>")]
pub struct RestartPlugin {
    pub plugin_id: String,
}

/// Message to get plugin status
#[derive(Message, Debug, Clone, Serialize, Deserialize)]
#[rtype(result = "A2AResult<PluginStatus>")]
pub struct GetPluginStatus {
    pub plugin_id: String,
}

/// Message to list all plugins
#[derive(Message, Debug)]
#[rtype(result = "Vec<PluginInfo>")]
pub struct ListPlugins;

/// Plugin information for listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub id: String,
    pub name: String,
    pub status: PluginStatus,
    pub grpc_port: u16,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub restart_count: u32,
}

/// Message to get supervisor statistics
#[derive(Message, Debug)]
#[rtype(result = "SupervisorStats")]
pub struct GetSupervisorStats;

/// Message for periodic health check
#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct PeriodicHealthCheck;

impl Actor for PluginSupervisorActor {
    type Context = Context<Self>;
    
    fn started(&mut self, ctx: &mut Self::Context) {
        info!("Plugin Supervisor Actor started");
        
        // Start periodic health checks
        self.start_health_monitoring(ctx);
        
        // Start periodic statistics update
        self.start_stats_update(ctx);
    }
    
    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("Plugin Supervisor Actor stopped");
        
        // Stop all plugins
        for (plugin_id, plugin) in &mut self.plugins {
            if let Some(ref mut process) = plugin.process {
                warn!("Forcefully stopping plugin {} during shutdown", plugin_id);
                let _ = process.kill();
            }
        }
    }
}

impl PluginSupervisorActor {
    /// Create a new Plugin Supervisor Actor
    pub fn new(config: SupervisorConfig) -> Self {
        Self {
            plugins: HashMap::new(),
            stats: SupervisorStats::default(),
            config,
        }
    }
    
    /// Start health monitoring
    fn start_health_monitoring(&self, ctx: &mut Context<Self>) {
        let interval = std::time::Duration::from_millis(self.config.health_check_interval_ms);
        
        ctx.run_interval(interval, |_actor, ctx| {
            ctx.address().do_send(PeriodicHealthCheck);
        });
    }
    
    /// Start statistics update
    fn start_stats_update(&self, ctx: &mut Context<Self>) {
        ctx.run_interval(
            std::time::Duration::from_secs(30),
            |actor, _ctx| {
                actor.update_stats();
                debug!("Supervisor stats: {:?}", actor.stats);
            }
        );
    }
    
    /// Update supervisor statistics
    fn update_stats(&mut self) {
        self.stats.total_plugins = self.plugins.len();
        self.stats.running_plugins = self.plugins.values()
            .filter(|p| p.status == PluginStatus::Running)
            .count();
        self.stats.failed_plugins = self.plugins.values()
            .filter(|p| p.status == PluginStatus::Failed)
            .count();
        self.stats.total_restarts = self.plugins.values()
            .map(|p| p.restart_count)
            .sum();
    }
    
    /// Start a plugin process
    fn start_plugin_process(&mut self, plugin_id: &str, executable_path: &str, grpc_port: u16, config: &HashMap<String, String>) -> A2AResult<()> {
        info!("Starting plugin process: {}", plugin_id);
        
        // Prepare environment variables
        let mut cmd = Command::new(executable_path);
        cmd.env("AGENTX_PLUGIN_ID", plugin_id)
           .env("AGENTX_GRPC_PORT", grpc_port.to_string())
           .stdin(Stdio::piped())
           .stdout(Stdio::piped())
           .stderr(Stdio::piped());
        
        // Add configuration as environment variables
        for (key, value) in config {
            cmd.env(format!("AGENTX_CONFIG_{}", key.to_uppercase()), value);
        }
        
        // Start the process
        match cmd.spawn() {
            Ok(process) => {
                let plugin_process = PluginProcess {
                    id: plugin_id.to_string(),
                    name: plugin_id.to_string(), // TODO: Get from config
                    executable_path: executable_path.to_string(),
                    process: Some(process),
                    grpc_port,
                    status: PluginStatus::Starting,
                    start_time: chrono::Utc::now(),
                    restart_count: 0,
                    last_health_check: None,
                };
                
                self.plugins.insert(plugin_id.to_string(), plugin_process);
                
                info!("Plugin {} started successfully on port {}", plugin_id, grpc_port);
                Ok(())
            }
            Err(e) => {
                error!("Failed to start plugin {}: {}", plugin_id, e);
                Err(A2AError::internal(format!("Failed to start plugin: {}", e)))
            }
        }
    }
    
    /// Stop a plugin process
    fn stop_plugin_process(&mut self, plugin_id: &str, force: bool) -> A2AResult<()> {
        info!("Stopping plugin process: {} (force: {})", plugin_id, force);
        
        if let Some(plugin) = self.plugins.get_mut(plugin_id) {
            plugin.status = PluginStatus::Stopping;
            
            if let Some(ref mut process) = plugin.process {
                if force {
                    match process.kill() {
                        Ok(_) => {
                            info!("Plugin {} forcefully stopped", plugin_id);
                            plugin.status = PluginStatus::Stopped;
                            plugin.process = None;
                        }
                        Err(e) => {
                            error!("Failed to kill plugin {}: {}", plugin_id, e);
                            plugin.status = PluginStatus::Failed;
                            return Err(A2AError::internal(format!("Failed to kill plugin: {}", e)));
                        }
                    }
                } else {
                    // Graceful shutdown - send SIGTERM and wait
                    match process.try_wait() {
                        Ok(Some(_)) => {
                            info!("Plugin {} already stopped", plugin_id);
                            plugin.status = PluginStatus::Stopped;
                            plugin.process = None;
                        }
                        Ok(None) => {
                            // Process is still running, terminate it
                            let _ = process.kill();
                            plugin.status = PluginStatus::Stopped;
                            plugin.process = None;
                        }
                        Err(e) => {
                            error!("Error checking plugin {} status: {}", plugin_id, e);
                            plugin.status = PluginStatus::Failed;
                        }
                    }
                }
            } else {
                plugin.status = PluginStatus::Stopped;
            }
            
            Ok(())
        } else {
            Err(A2AError::agent_not_found(plugin_id))
        }
    }
    
    /// Check plugin health
    fn check_plugin_health(&mut self, plugin_id: &str) -> bool {
        if let Some(plugin) = self.plugins.get_mut(plugin_id) {
            if let Some(ref mut process) = plugin.process {
                match process.try_wait() {
                    Ok(Some(exit_status)) => {
                        warn!("Plugin {} exited with status: {:?}", plugin_id, exit_status);
                        plugin.status = PluginStatus::Failed;
                        plugin.process = None;
                        false
                    }
                    Ok(None) => {
                        // Process is still running
                        plugin.status = PluginStatus::Running;
                        plugin.last_health_check = Some(chrono::Utc::now());
                        true
                    }
                    Err(e) => {
                        error!("Error checking plugin {} health: {}", plugin_id, e);
                        plugin.status = PluginStatus::Failed;
                        false
                    }
                }
            } else {
                plugin.status = PluginStatus::Stopped;
                false
            }
        } else {
            false
        }
    }
}

/// Handle StartPlugin
impl Handler<StartPlugin> for PluginSupervisorActor {
    type Result = A2AResult<()>;
    
    fn handle(&mut self, msg: StartPlugin, _ctx: &mut Self::Context) -> Self::Result {
        self.start_plugin_process(&msg.plugin_id, &msg.executable_path, msg.grpc_port, &msg.config)
    }
}

/// Handle StopPlugin
impl Handler<StopPlugin> for PluginSupervisorActor {
    type Result = A2AResult<()>;
    
    fn handle(&mut self, msg: StopPlugin, _ctx: &mut Self::Context) -> Self::Result {
        self.stop_plugin_process(&msg.plugin_id, msg.force)
    }
}

/// Handle RestartPlugin
impl Handler<RestartPlugin> for PluginSupervisorActor {
    type Result = A2AResult<()>;
    
    fn handle(&mut self, msg: RestartPlugin, ctx: &mut Self::Context) -> Self::Result {
        let plugin_id = msg.plugin_id.clone();
        
        if let Some(plugin) = self.plugins.get(&plugin_id).cloned() {
            // Stop the plugin first
            self.stop_plugin_process(&plugin_id, true)?;
            
            // Increment restart count
            if let Some(plugin_ref) = self.plugins.get_mut(&plugin_id) {
                plugin_ref.restart_count += 1;
            }
            
            // Schedule restart after delay
            let restart_delay = std::time::Duration::from_millis(self.config.restart_delay_ms);
            let executable_path = plugin.executable_path.clone();
            let grpc_port = plugin.grpc_port;
            let config = HashMap::new(); // TODO: Store original config
            
            ctx.run_later(restart_delay, move |actor, _ctx| {
                if let Err(e) = actor.start_plugin_process(&plugin_id, &executable_path, grpc_port, &config) {
                    error!("Failed to restart plugin {}: {}", plugin_id, e);
                }
            });
            
            Ok(())
        } else {
            Err(A2AError::agent_not_found(&plugin_id))
        }
    }
}

/// Handle GetPluginStatus
impl Handler<GetPluginStatus> for PluginSupervisorActor {
    type Result = A2AResult<PluginStatus>;
    
    fn handle(&mut self, msg: GetPluginStatus, _ctx: &mut Self::Context) -> Self::Result {
        if let Some(plugin) = self.plugins.get(&msg.plugin_id) {
            Ok(plugin.status.clone())
        } else {
            Err(A2AError::agent_not_found(&msg.plugin_id))
        }
    }
}

/// Handle ListPlugins
impl Handler<ListPlugins> for PluginSupervisorActor {
    type Result = Vec<PluginInfo>;
    
    fn handle(&mut self, _msg: ListPlugins, _ctx: &mut Self::Context) -> Self::Result {
        self.plugins.values().map(|plugin| {
            PluginInfo {
                id: plugin.id.clone(),
                name: plugin.name.clone(),
                status: plugin.status.clone(),
                grpc_port: plugin.grpc_port,
                start_time: plugin.start_time,
                restart_count: plugin.restart_count,
            }
        }).collect()
    }
}

/// Handle GetSupervisorStats
impl Handler<GetSupervisorStats> for PluginSupervisorActor {
    type Result = SupervisorStats;
    
    fn handle(&mut self, _msg: GetSupervisorStats, _ctx: &mut Self::Context) -> Self::Result {
        self.stats.clone()
    }
}

/// Handle PeriodicHealthCheck
impl Handler<PeriodicHealthCheck> for PluginSupervisorActor {
    type Result = ();
    
    fn handle(&mut self, _msg: PeriodicHealthCheck, ctx: &mut Self::Context) -> Self::Result {
        debug!("Performing periodic health check");
        
        let plugin_ids: Vec<String> = self.plugins.keys().cloned().collect();
        
        for plugin_id in plugin_ids {
            let is_healthy = self.check_plugin_health(&plugin_id);
            
            if !is_healthy {
                // Check if we should restart the plugin
                if let Some(plugin) = self.plugins.get(&plugin_id) {
                    if plugin.restart_count < self.config.max_restart_attempts {
                        warn!("Plugin {} is unhealthy, scheduling restart", plugin_id);
                        ctx.address().do_send(RestartPlugin { plugin_id });
                    } else {
                        error!("Plugin {} has exceeded max restart attempts", plugin_id);
                    }
                }
            }
        }
        
        self.stats.health_checks_performed += 1;
        self.update_stats();
    }
}

impl Default for SupervisorConfig {
    fn default() -> Self {
        Self {
            max_restart_attempts: 3,
            restart_delay_ms: 5000,      // 5 seconds
            health_check_interval_ms: 30000, // 30 seconds
            startup_timeout_ms: 10000,   // 10 seconds
            shutdown_timeout_ms: 5000,   // 5 seconds
        }
    }
}
