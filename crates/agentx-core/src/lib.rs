//! AgentX核心库
//! 
//! 提供AgentX平台的核心功能，包括协议兼容、云原生部署和开发者生态系统支持

pub mod protocol_compat;
pub mod cloud_native;
pub mod helm_charts;
pub mod cicd_pipeline;
pub mod developer_ecosystem;
pub mod debug_diagnostics;
pub mod performance_analyzer;
pub mod real_network_benchmarks;
pub mod distributed_tracing;
pub mod error_recovery;

// 重新导出主要类型
pub use protocol_compat::{ProtocolCompatManager, mcp, openai};
pub use cloud_native::{CloudNativeManager, KubernetesConfig, DockerConfig, CloudProviderConfig};
pub use developer_ecosystem::{DeveloperEcosystemManager, PluginMarketManager, CliToolManager};
pub use debug_diagnostics::{DebugDiagnosticsManager, DiagnosticsConfig, SystemDiagnosticsReport};
pub use performance_analyzer::{PerformanceAnalyzer, PerformanceConfig, BenchmarkResult};
pub use real_network_benchmarks::{RealNetworkBenchmarks, RealNetworkConfig, RealNetworkBenchmarkResult};
pub use distributed_tracing::{DistributedTracingManager, TracingConfig, TraceSession, Span, TraceStatus, SpanStatus};
pub use error_recovery::{ErrorRecoveryManager, ErrorRecoveryConfig, ComponentStatus, ErrorType, RecoveryStrategy};

/// AgentX核心版本
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// AgentX核心功能管理器
pub struct AgentXCore {
    protocol_compat: ProtocolCompatManager,
    cloud_native: CloudNativeManager,
    developer_ecosystem: DeveloperEcosystemManager,
    error_recovery: ErrorRecoveryManager,
}

impl AgentXCore {
    /// 创建新的AgentX核心实例
    pub fn new() -> Self {
        let error_recovery_config = ErrorRecoveryConfig::default();
        Self {
            protocol_compat: ProtocolCompatManager::new(),
            cloud_native: CloudNativeManager::new(),
            developer_ecosystem: DeveloperEcosystemManager::new(),
            error_recovery: ErrorRecoveryManager::new(error_recovery_config),
        }
    }
    
    /// 获取协议兼容管理器
    pub fn protocol_compat(&mut self) -> &mut ProtocolCompatManager {
        &mut self.protocol_compat
    }
    
    /// 获取云原生管理器
    pub fn cloud_native(&mut self) -> &mut CloudNativeManager {
        &mut self.cloud_native
    }
    
    /// 获取开发者生态系统管理器
    pub fn developer_ecosystem(&mut self) -> &mut DeveloperEcosystemManager {
        &mut self.developer_ecosystem
    }

    /// 获取错误恢复管理器
    pub fn error_recovery(&self) -> &ErrorRecoveryManager {
        &self.error_recovery
    }
    
    /// 初始化AgentX核心
    pub async fn initialize(&mut self) -> agentx_a2a::A2AResult<()> {
        // 初始化各个子系统
        self.developer_ecosystem.setup_developer_environment().await?;

        // 启动错误恢复管理器
        if let Err(e) = self.error_recovery.start().await {
            eprintln!("⚠️ 错误恢复管理器启动失败: {}", e);
        }

        // 注册核心组件到错误恢复管理器
        self.error_recovery.register_component("protocol_compat", RecoveryStrategy::Retry).await;
        self.error_recovery.register_component("cloud_native", RecoveryStrategy::Restart).await;
        self.error_recovery.register_component("developer_ecosystem", RecoveryStrategy::Degrade).await;

        println!("✅ AgentX核心初始化完成");
        Ok(())
    }
    
    /// 获取系统信息
    pub fn get_system_info(&self) -> SystemInfo {
        SystemInfo {
            version: VERSION.to_string(),
            features: vec![
                "协议兼容层".to_string(),
                "云原生部署".to_string(),
                "开发者生态系统".to_string(),
                "插件市场".to_string(),
                "CLI工具".to_string(),
                "错误恢复和故障处理".to_string(),
                "断路器模式".to_string(),
                "自动重试机制".to_string(),
            ],
        }
    }
}

impl Default for AgentXCore {
    fn default() -> Self {
        Self::new()
    }
}

/// 系统信息
#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub version: String,
    pub features: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_agentx_core_initialization() {
        let mut core = AgentXCore::new();
        
        let result = core.initialize().await;
        assert!(result.is_ok());
        
        let info = core.get_system_info();
        assert_eq!(info.version, VERSION);
        assert!(!info.features.is_empty());
    }
    
    #[test]
    fn test_system_info() {
        let core = AgentXCore::new();
        let info = core.get_system_info();
        
        assert!(info.features.contains(&"协议兼容层".to_string()));
        assert!(info.features.contains(&"云原生部署".to_string()));
        assert!(info.features.contains(&"开发者生态系统".to_string()));
    }
}
