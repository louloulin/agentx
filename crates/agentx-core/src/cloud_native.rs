//! 云原生部署支持
//! 
//! 提供Kubernetes、Docker和多云部署的支持

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use agentx_a2a::A2AResult;

/// Kubernetes部署配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KubernetesConfig {
    pub namespace: String,
    pub deployment_name: String,
    pub service_name: String,
    pub replicas: u32,
    pub image: String,
    pub image_tag: String,
    pub resources: ResourceRequirements,
    pub env_vars: HashMap<String, String>,
    pub config_maps: Vec<String>,
    pub secrets: Vec<String>,
    pub ingress: Option<IngressConfig>,
}

/// 资源需求配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub cpu_request: String,
    pub cpu_limit: String,
    pub memory_request: String,
    pub memory_limit: String,
}

/// Ingress配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngressConfig {
    pub host: String,
    pub path: String,
    pub tls_enabled: bool,
    pub cert_manager: bool,
}

/// Docker配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerConfig {
    pub image_name: String,
    pub tag: String,
    pub dockerfile_path: String,
    pub build_context: String,
    pub build_args: HashMap<String, String>,
    pub ports: Vec<PortMapping>,
    pub volumes: Vec<VolumeMapping>,
    pub environment: HashMap<String, String>,
}

/// 端口映射
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortMapping {
    pub host_port: u16,
    pub container_port: u16,
    pub protocol: String,
}

/// 卷映射
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeMapping {
    pub host_path: String,
    pub container_path: String,
    pub read_only: bool,
}

/// 云提供商配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudProviderConfig {
    pub provider: CloudProvider,
    pub region: String,
    pub credentials: CloudCredentials,
    pub networking: NetworkConfig,
    pub storage: StorageConfig,
}

/// 云提供商类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CloudProvider {
    AWS,
    Azure,
    GCP,
    AliCloud,
    TencentCloud,
    DigitalOcean,
    Linode,
}

/// 云凭证配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudCredentials {
    pub access_key: Option<String>,
    pub secret_key: Option<String>,
    pub token: Option<String>,
    pub service_account: Option<String>,
}

/// 网络配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub vpc_id: Option<String>,
    pub subnet_ids: Vec<String>,
    pub security_groups: Vec<String>,
    pub load_balancer: Option<LoadBalancerConfig>,
}

/// 负载均衡器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancerConfig {
    pub r#type: String,
    pub scheme: String,
    pub listeners: Vec<ListenerConfig>,
}

/// 监听器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListenerConfig {
    pub port: u16,
    pub protocol: String,
    pub ssl_cert: Option<String>,
}

/// 存储配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub persistent_volumes: Vec<PersistentVolumeConfig>,
    pub object_storage: Option<ObjectStorageConfig>,
}

/// 持久卷配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentVolumeConfig {
    pub name: String,
    pub size: String,
    pub storage_class: String,
    pub access_modes: Vec<String>,
}

/// 对象存储配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectStorageConfig {
    pub bucket_name: String,
    pub endpoint: Option<String>,
    pub region: String,
}

/// Kubernetes部署管理器
pub struct KubernetesDeploymentManager {
    config: KubernetesConfig,
}

impl KubernetesDeploymentManager {
    pub fn new(config: KubernetesConfig) -> Self {
        Self { config }
    }
    
    /// 生成Kubernetes Deployment YAML
    pub fn generate_deployment_yaml(&self) -> A2AResult<String> {
        let yaml = format!(r#"
apiVersion: apps/v1
kind: Deployment
metadata:
  name: {}
  namespace: {}
  labels:
    app: agentx
    component: core
spec:
  replicas: {}
  selector:
    matchLabels:
      app: agentx
      component: core
  template:
    metadata:
      labels:
        app: agentx
        component: core
    spec:
      containers:
      - name: agentx-core
        image: {}:{}
        ports:
        - containerPort: 50051
          name: grpc
        - containerPort: 8080
          name: http
        env:
{}
        resources:
          requests:
            cpu: {}
            memory: {}
          limits:
            cpu: {}
            memory: {}
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
"#,
            self.config.deployment_name,
            self.config.namespace,
            self.config.replicas,
            self.config.image,
            self.config.image_tag,
            self.generate_env_vars(),
            self.config.resources.cpu_request,
            self.config.resources.memory_request,
            self.config.resources.cpu_limit,
            self.config.resources.memory_limit,
        );
        
        Ok(yaml)
    }
    
    /// 生成Kubernetes Service YAML
    pub fn generate_service_yaml(&self) -> A2AResult<String> {
        let yaml = format!(r#"
apiVersion: v1
kind: Service
metadata:
  name: {}
  namespace: {}
  labels:
    app: agentx
    component: core
spec:
  selector:
    app: agentx
    component: core
  ports:
  - name: grpc
    port: 50051
    targetPort: 50051
    protocol: TCP
  - name: http
    port: 8080
    targetPort: 8080
    protocol: TCP
  type: ClusterIP
"#,
            self.config.service_name,
            self.config.namespace,
        );
        
        Ok(yaml)
    }
    
    /// 生成Ingress YAML
    pub fn generate_ingress_yaml(&self) -> A2AResult<Option<String>> {
        if let Some(ingress) = &self.config.ingress {
            let yaml = format!(r#"
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: {}-ingress
  namespace: {}
  annotations:
    nginx.ingress.kubernetes.io/rewrite-target: /
{}
spec:
{}
  rules:
  - host: {}
    http:
      paths:
      - path: {}
        pathType: Prefix
        backend:
          service:
            name: {}
            port:
              number: 8080
"#,
                self.config.deployment_name,
                self.config.namespace,
                if ingress.cert_manager {
                    "    cert-manager.io/cluster-issuer: letsencrypt-prod"
                } else {
                    ""
                },
                if ingress.tls_enabled {
                    format!("  tls:\n  - hosts:\n    - {}\n    secretName: {}-tls",
                        ingress.host, self.config.deployment_name)
                } else {
                    String::new()
                },
                ingress.host,
                ingress.path,
                self.config.service_name,
            );
            
            Ok(Some(yaml))
        } else {
            Ok(None)
        }
    }
    
    fn generate_env_vars(&self) -> String {
        self.config.env_vars
            .iter()
            .map(|(key, value)| format!("        - name: {}\n          value: \"{}\"", key, value))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// Docker部署管理器
pub struct DockerDeploymentManager {
    config: DockerConfig,
}

impl DockerDeploymentManager {
    pub fn new(config: DockerConfig) -> Self {
        Self { config }
    }
    
    /// 生成Dockerfile
    pub fn generate_dockerfile(&self) -> A2AResult<String> {
        let dockerfile = r#"
# 多阶段构建
FROM rust:1.75 as builder

WORKDIR /app
COPY . .

# 构建应用
RUN cargo build --release

# 运行时镜像
FROM debian:bookworm-slim

# 安装运行时依赖
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# 创建应用用户
RUN useradd -r -s /bin/false agentx

# 复制二进制文件
COPY --from=builder /app/target/release/agentx-core /usr/local/bin/agentx-core

# 设置权限
RUN chmod +x /usr/local/bin/agentx-core

# 切换到应用用户
USER agentx

# 暴露端口
EXPOSE 50051 8080

# 健康检查
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# 启动命令
CMD ["agentx-core"]
"#;
        
        Ok(dockerfile.to_string())
    }
    
    /// 生成docker-compose.yml
    pub fn generate_docker_compose(&self) -> A2AResult<String> {
        let compose = format!(r#"
version: '3.8'

services:
  agentx-core:
    build:
      context: {}
      dockerfile: {}
    image: {}:{}
    container_name: agentx-core
    restart: unless-stopped
    ports:
{}
    environment:
{}
    volumes:
{}
    networks:
      - agentx-network
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s

networks:
  agentx-network:
    driver: bridge

volumes:
  agentx-data:
    driver: local
"#,
            self.config.build_context,
            self.config.dockerfile_path,
            self.config.image_name,
            self.config.tag,
            self.generate_port_mappings(),
            self.generate_environment_vars(),
            self.generate_volume_mappings(),
        );
        
        Ok(compose)
    }
    
    fn generate_port_mappings(&self) -> String {
        self.config.ports
            .iter()
            .map(|port| format!("      - \"{}:{}\"", port.host_port, port.container_port))
            .collect::<Vec<_>>()
            .join("\n")
    }
    
    fn generate_environment_vars(&self) -> String {
        self.config.environment
            .iter()
            .map(|(key, value)| format!("      {}: \"{}\"", key, value))
            .collect::<Vec<_>>()
            .join("\n")
    }
    
    fn generate_volume_mappings(&self) -> String {
        self.config.volumes
            .iter()
            .map(|vol| format!("      - \"{}:{}{}\"", 
                vol.host_path, 
                vol.container_path,
                if vol.read_only { ":ro" } else { "" }
            ))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// 云原生部署管理器
pub struct CloudNativeManager {
    kubernetes: Option<KubernetesDeploymentManager>,
    docker: Option<DockerDeploymentManager>,
    cloud_config: Option<CloudProviderConfig>,
}

impl CloudNativeManager {
    pub fn new() -> Self {
        Self {
            kubernetes: None,
            docker: None,
            cloud_config: None,
        }
    }
    
    /// 设置Kubernetes配置
    pub fn with_kubernetes(mut self, config: KubernetesConfig) -> Self {
        self.kubernetes = Some(KubernetesDeploymentManager::new(config));
        self
    }
    
    /// 设置Docker配置
    pub fn with_docker(mut self, config: DockerConfig) -> Self {
        self.docker = Some(DockerDeploymentManager::new(config));
        self
    }
    
    /// 设置云提供商配置
    pub fn with_cloud_provider(mut self, config: CloudProviderConfig) -> Self {
        self.cloud_config = Some(config);
        self
    }
    
    /// 生成所有部署文件
    pub async fn generate_deployment_files(&self) -> A2AResult<HashMap<String, String>> {
        let mut files = HashMap::new();
        
        // 生成Kubernetes文件
        if let Some(k8s) = &self.kubernetes {
            files.insert("deployment.yaml".to_string(), k8s.generate_deployment_yaml()?);
            files.insert("service.yaml".to_string(), k8s.generate_service_yaml()?);
            
            if let Some(ingress) = k8s.generate_ingress_yaml()? {
                files.insert("ingress.yaml".to_string(), ingress);
            }
        }
        
        // 生成Docker文件
        if let Some(docker) = &self.docker {
            files.insert("Dockerfile".to_string(), docker.generate_dockerfile()?);
            files.insert("docker-compose.yml".to_string(), docker.generate_docker_compose()?);
        }
        
        Ok(files)
    }
    
    /// 验证部署配置
    pub async fn validate_configuration(&self) -> A2AResult<Vec<String>> {
        let mut warnings = Vec::new();
        
        if self.kubernetes.is_none() && self.docker.is_none() {
            warnings.push("未配置任何部署方式".to_string());
        }
        
        if let Some(cloud_config) = &self.cloud_config {
            if cloud_config.credentials.access_key.is_none() {
                warnings.push("云提供商凭证不完整".to_string());
            }
        }
        
        Ok(warnings)
    }
}

impl Default for CloudNativeManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ResourceRequirements {
    fn default() -> Self {
        Self {
            cpu_request: "100m".to_string(),
            cpu_limit: "500m".to_string(),
            memory_request: "128Mi".to_string(),
            memory_limit: "512Mi".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_kubernetes_deployment() {
        let config = KubernetesConfig {
            namespace: "agentx".to_string(),
            deployment_name: "agentx-core".to_string(),
            service_name: "agentx-service".to_string(),
            replicas: 3,
            image: "agentx/core".to_string(),
            image_tag: "latest".to_string(),
            resources: ResourceRequirements::default(),
            env_vars: {
                let mut env = HashMap::new();
                env.insert("RUST_LOG".to_string(), "info".to_string());
                env
            },
            config_maps: vec![],
            secrets: vec![],
            ingress: None,
        };
        
        let manager = KubernetesDeploymentManager::new(config);
        let deployment_yaml = manager.generate_deployment_yaml().unwrap();
        
        assert!(deployment_yaml.contains("agentx-core"));
        assert!(deployment_yaml.contains("replicas: 3"));
    }
    
    #[tokio::test]
    async fn test_docker_deployment() {
        let config = DockerConfig {
            image_name: "agentx/core".to_string(),
            tag: "latest".to_string(),
            dockerfile_path: "Dockerfile".to_string(),
            build_context: ".".to_string(),
            build_args: HashMap::new(),
            ports: vec![
                PortMapping {
                    host_port: 50051,
                    container_port: 50051,
                    protocol: "tcp".to_string(),
                }
            ],
            volumes: vec![],
            environment: HashMap::new(),
        };
        
        let manager = DockerDeploymentManager::new(config);
        let dockerfile = manager.generate_dockerfile().unwrap();
        let compose = manager.generate_docker_compose().unwrap();
        
        assert!(dockerfile.contains("FROM rust:1.75"));
        assert!(compose.contains("agentx-core"));
    }
    
    #[tokio::test]
    async fn test_cloud_native_manager() {
        let mut manager = CloudNativeManager::new();
        
        let k8s_config = KubernetesConfig {
            namespace: "agentx".to_string(),
            deployment_name: "agentx-core".to_string(),
            service_name: "agentx-service".to_string(),
            replicas: 2,
            image: "agentx/core".to_string(),
            image_tag: "v1.0.0".to_string(),
            resources: ResourceRequirements::default(),
            env_vars: HashMap::new(),
            config_maps: vec![],
            secrets: vec![],
            ingress: None,
        };
        
        manager = manager.with_kubernetes(k8s_config);
        
        let files = manager.generate_deployment_files().await.unwrap();
        assert!(files.contains_key("deployment.yaml"));
        assert!(files.contains_key("service.yaml"));
        
        let warnings = manager.validate_configuration().await.unwrap();
        assert!(warnings.is_empty());
    }
}
