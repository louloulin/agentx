//! 云原生部署功能集成测试
//! 
//! 测试Kubernetes部署、Docker镜像构建、Helm Charts、CI/CD流水线等功能

use agentx_core::cloud_native::*;
use agentx_core::helm_charts::*;
use agentx_core::cicd_pipeline::*;
use std::collections::HashMap;

#[tokio::test]
async fn test_kubernetes_deployment_generation() {
    println!("🚀 测试Kubernetes部署配置生成");

    let mut env_vars = HashMap::new();
    env_vars.insert("RUST_LOG".to_string(), "info".to_string());
    env_vars.insert("AGENTX_PORT".to_string(), "50051".to_string());

    let config = KubernetesConfig {
        namespace: "agentx-system".to_string(),
        deployment_name: "agentx-core".to_string(),
        service_name: "agentx-service".to_string(),
        replicas: 3,
        image: "ghcr.io/agentx/core".to_string(),
        image_tag: "v1.0.0".to_string(),
        resources: ResourceRequirements {
            cpu_request: "200m".to_string(),
            cpu_limit: "1000m".to_string(),
            memory_request: "256Mi".to_string(),
            memory_limit: "1Gi".to_string(),
        },
        env_vars,
        config_maps: vec!["agentx-config".to_string()],
        secrets: vec!["agentx-secrets".to_string()],
        ingress: Some(IngressConfig {
            host: "agentx.example.com".to_string(),
            path: "/".to_string(),
            tls_enabled: true,
            cert_manager: true,
        }),
    };

    let manager = KubernetesDeploymentManager::new(config);

    // 测试Deployment YAML生成
    let deployment_yaml = manager.generate_deployment_yaml().unwrap();
    assert!(deployment_yaml.contains("agentx-core"));
    assert!(deployment_yaml.contains("replicas: 3"));
    assert!(deployment_yaml.contains("ghcr.io/agentx/core:v1.0.0"));
    assert!(deployment_yaml.contains("cpu: 200m"));
    assert!(deployment_yaml.contains("memory: 256Mi"));
    assert!(deployment_yaml.contains("RUST_LOG"));

    // 测试Service YAML生成
    let service_yaml = manager.generate_service_yaml().unwrap();
    assert!(service_yaml.contains("agentx-service"));
    assert!(service_yaml.contains("port: 50051"));
    assert!(service_yaml.contains("port: 8080"));
    assert!(service_yaml.contains("ClusterIP"));

    // 测试Ingress YAML生成
    let ingress_yaml = manager.generate_ingress_yaml().unwrap();
    assert!(ingress_yaml.is_some());
    let ingress = ingress_yaml.unwrap();
    assert!(ingress.contains("agentx.example.com"));
    assert!(ingress.contains("cert-manager.io/cluster-issuer"));
    assert!(ingress.contains("tls:"));

    println!("✅ Kubernetes部署配置生成测试通过");
}

#[tokio::test]
async fn test_docker_deployment_generation() {
    println!("🚀 测试Docker部署配置生成");

    let mut env_vars = HashMap::new();
    env_vars.insert("RUST_LOG".to_string(), "debug".to_string());

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
            },
            PortMapping {
                host_port: 8080,
                container_port: 8080,
                protocol: "tcp".to_string(),
            },
        ],
        volumes: vec![
            VolumeMapping {
                host_path: "./data".to_string(),
                container_path: "/app/data".to_string(),
                read_only: false,
            },
        ],
        environment: env_vars,
    };

    let manager = DockerDeploymentManager::new(config);

    // 测试Dockerfile生成
    let dockerfile = manager.generate_dockerfile().unwrap();
    assert!(dockerfile.contains("FROM rust:1.75"));
    assert!(dockerfile.contains("FROM debian:bookworm-slim"));
    assert!(dockerfile.contains("EXPOSE 50051 8080"));
    assert!(dockerfile.contains("HEALTHCHECK"));
    assert!(dockerfile.contains("agentx-core"));

    // 测试docker-compose.yml生成
    let compose = manager.generate_docker_compose().unwrap();
    assert!(compose.contains("version: '3.8'"));
    assert!(compose.contains("agentx-core"));
    assert!(compose.contains("50051:50051"));
    assert!(compose.contains("8080:8080"));
    assert!(compose.contains("RUST_LOG: \"debug\""));
    assert!(compose.contains("./data:/app/data"));
    assert!(compose.contains("healthcheck:"));

    println!("✅ Docker部署配置生成测试通过");
}

#[tokio::test]
async fn test_helm_charts_generation() {
    println!("🚀 测试Helm Charts生成");

    let config = HelmChartConfig {
        name: "agentx".to_string(),
        version: "1.0.0".to_string(),
        app_version: "1.0.0".to_string(),
        description: "AgentX - Universal AI Agent Framework".to_string(),
        keywords: vec![
            "ai".to_string(),
            "agent".to_string(),
            "framework".to_string(),
            "rust".to_string(),
        ],
        maintainers: vec![
            Maintainer {
                name: "AgentX Team".to_string(),
                email: "team@agentx.dev".to_string(),
                url: Some("https://agentx.dev".to_string()),
            }
        ],
        dependencies: vec![
            Dependency {
                name: "postgresql".to_string(),
                version: "12.1.0".to_string(),
                repository: "https://charts.bitnami.com/bitnami".to_string(),
                condition: Some("postgresql.enabled".to_string()),
                tags: vec!["database".to_string()],
            }
        ],
        default_values: HelmValues {
            replica_count: 2,
            image: ImageConfig {
                repository: "ghcr.io/agentx/core".to_string(),
                pull_policy: "IfNotPresent".to_string(),
                tag: "1.0.0".to_string(),
                pull_secrets: vec![],
            },
            autoscaling: AutoscalingConfig {
                enabled: true,
                min_replicas: 2,
                max_replicas: 10,
                target_cpu_utilization: 70,
                target_memory_utilization: Some(80),
            },
            ..Default::default()
        },
    };

    let generator = HelmChartsGenerator::new(config);

    // 测试Chart.yaml生成
    let chart_yaml = generator.generate_chart_yaml().unwrap();
    assert!(chart_yaml.contains("name: agentx"));
    assert!(chart_yaml.contains("version: 1.0.0"));
    assert!(chart_yaml.contains("appVersion: \"1.0.0\""));
    assert!(chart_yaml.contains("AgentX Team"));
    assert!(chart_yaml.contains("postgresql"));

    // 测试values.yaml生成
    let values_yaml = generator.generate_values_yaml().unwrap();
    assert!(values_yaml.contains("replicaCount: 2"));
    assert!(values_yaml.contains("repository: ghcr.io/agentx/core"));
    assert!(values_yaml.contains("tag: \"1.0.0\""));
    assert!(values_yaml.contains("enabled: true"));
    assert!(values_yaml.contains("minReplicas: 2"));
    assert!(values_yaml.contains("maxReplicas: 10"));

    // 测试deployment模板生成
    let deployment_template = generator.generate_deployment_template().unwrap();
    assert!(deployment_template.contains("apiVersion: apps/v1"));
    assert!(deployment_template.contains("kind: Deployment"));
    assert!(deployment_template.contains("{{ include \"agentx.fullname\" . }}"));
    assert!(deployment_template.contains("containerPort: 50051"));
    assert!(deployment_template.contains("livenessProbe:"));
    assert!(deployment_template.contains("readinessProbe:"));

    // 测试service模板生成
    let service_template = generator.generate_service_template().unwrap();
    assert!(service_template.contains("apiVersion: v1"));
    assert!(service_template.contains("kind: Service"));
    assert!(service_template.contains("{{ .Values.service.type }}"));

    // 测试ingress模板生成
    let ingress_template = generator.generate_ingress_template().unwrap();
    assert!(ingress_template.contains("{{- if .Values.ingress.enabled -}}"));
    assert!(ingress_template.contains("kind: Ingress"));

    // 测试所有文件生成
    let all_files = generator.generate_all_files().unwrap();
    assert_eq!(all_files.len(), 8);
    assert!(all_files.contains_key("Chart.yaml"));
    assert!(all_files.contains_key("values.yaml"));
    assert!(all_files.contains_key("templates/deployment.yaml"));
    assert!(all_files.contains_key("templates/service.yaml"));
    assert!(all_files.contains_key("templates/ingress.yaml"));
    assert!(all_files.contains_key("templates/_helpers.tpl"));
    assert!(all_files.contains_key("templates/hpa.yaml"));
    assert!(all_files.contains_key("templates/serviceaccount.yaml"));

    println!("✅ Helm Charts生成测试通过");
}

#[tokio::test]
async fn test_cicd_pipeline_generation() {
    println!("🚀 测试CI/CD流水线生成");

    let mut env_vars = HashMap::new();
    env_vars.insert("RUST_LOG".to_string(), "info".to_string());

    let config = CICDConfig {
        platform: CICDPlatform::GitHubActions,
        project_name: "agentx".to_string(),
        build_environment: BuildEnvironment {
            rust_version: "1.75".to_string(),
            os: vec!["ubuntu-latest".to_string(), "windows-latest".to_string(), "macos-latest".to_string()],
            arch: vec!["x86_64".to_string(), "aarch64".to_string()],
            cache_enabled: true,
            build_tools: vec!["cargo".to_string()],
        },
        test_config: TestConfig {
            unit_tests: true,
            integration_tests: true,
            performance_tests: true,
            coverage_enabled: true,
            coverage_threshold: 85.0,
            timeout_minutes: 45,
        },
        deployment_config: DeploymentConfig {
            environments: vec![
                Environment {
                    name: "staging".to_string(),
                    branch: "develop".to_string(),
                    auto_deploy: true,
                    approval_required: false,
                    env_vars: HashMap::new(),
                },
                Environment {
                    name: "production".to_string(),
                    branch: "main".to_string(),
                    auto_deploy: false,
                    approval_required: true,
                    env_vars: HashMap::new(),
                },
            ],
            docker: DockerBuildConfig {
                image_name: "agentx/core".to_string(),
                registry: "ghcr.io".to_string(),
                build_args: HashMap::new(),
                multi_platform: true,
                platforms: vec![
                    "linux/amd64".to_string(),
                    "linux/arm64".to_string(),
                ],
            },
            kubernetes: Some(K8sDeployConfig {
                cluster: "production-cluster".to_string(),
                namespace: "agentx-system".to_string(),
                helm_chart_path: "./helm/agentx".to_string(),
                values_files: vec!["values.yaml".to_string(), "values-prod.yaml".to_string()],
            }),
            artifact_registry: ArtifactRegistry {
                registry_type: "docker".to_string(),
                url: "ghcr.io".to_string(),
                auth_config: "github".to_string(),
            },
        },
        environment_variables: env_vars,
        secrets: vec![
            "REGISTRY_USERNAME".to_string(),
            "REGISTRY_PASSWORD".to_string(),
            "KUBECONFIG".to_string(),
        ],
        triggers: TriggerConfig {
            on_push: true,
            on_pull_request: true,
            on_schedule: Some("0 2 * * *".to_string()),
            on_tag: true,
            branch_filters: vec!["main".to_string(), "develop".to_string()],
        },
    };

    let generator = CICDPipelineGenerator::new(config);

    // 测试GitHub Actions流水线生成
    let pipeline_files = generator.generate_pipeline().unwrap();
    assert_eq!(pipeline_files.len(), 3);
    assert!(pipeline_files.contains_key(".github/workflows/ci.yml"));
    assert!(pipeline_files.contains_key(".github/workflows/cd.yml"));
    assert!(pipeline_files.contains_key(".github/workflows/release.yml"));

    // 验证CI工作流内容
    let ci_workflow = pipeline_files.get(".github/workflows/ci.yml").unwrap();
    assert!(ci_workflow.contains("name: CI"));
    assert!(ci_workflow.contains("cargo fmt --all -- --check"));
    assert!(ci_workflow.contains("cargo clippy"));
    assert!(ci_workflow.contains("cargo test"));
    assert!(ci_workflow.contains("timeout-minutes: 45"));
    assert!(ci_workflow.contains("cargo-tarpaulin"));
    assert!(ci_workflow.contains("performance tests"));

    // 验证CD工作流内容
    let cd_workflow = pipeline_files.get(".github/workflows/cd.yml").unwrap();
    assert!(cd_workflow.contains("name: CD"));
    assert!(cd_workflow.contains("docker/build-push-action"));
    assert!(cd_workflow.contains("linux/amd64,linux/arm64"));
    assert!(cd_workflow.contains("environment: staging"));
    assert!(cd_workflow.contains("environment: production"));

    // 验证Release工作流内容
    let release_workflow = pipeline_files.get(".github/workflows/release.yml").unwrap();
    assert!(release_workflow.contains("name: Release"));
    assert!(release_workflow.contains("create-release"));
    assert!(release_workflow.contains("x86_64-unknown-linux-gnu"));
    assert!(release_workflow.contains("x86_64-pc-windows-msvc"));
    assert!(release_workflow.contains("x86_64-apple-darwin"));

    println!("✅ CI/CD流水线生成测试通过");
}

#[tokio::test]
async fn test_multiple_cicd_platforms() {
    println!("🚀 测试多种CI/CD平台支持");

    let platforms = vec![
        CICDPlatform::GitHubActions,
        CICDPlatform::GitLabCI,
        CICDPlatform::Jenkins,
        CICDPlatform::AzureDevOps,
        CICDPlatform::CircleCI,
    ];

    for platform in platforms {
        let config = CICDConfig {
            platform: platform.clone(),
            ..Default::default()
        };

        let generator = CICDPipelineGenerator::new(config);
        let pipeline_files = generator.generate_pipeline().unwrap();

        assert!(!pipeline_files.is_empty());

        match platform {
            CICDPlatform::GitHubActions => {
                assert!(pipeline_files.contains_key(".github/workflows/ci.yml"));
            }
            CICDPlatform::GitLabCI => {
                assert!(pipeline_files.contains_key(".gitlab-ci.yml"));
            }
            CICDPlatform::Jenkins => {
                assert!(pipeline_files.contains_key("Jenkinsfile"));
            }
            CICDPlatform::AzureDevOps => {
                assert!(pipeline_files.contains_key("azure-pipelines.yml"));
            }
            CICDPlatform::CircleCI => {
                assert!(pipeline_files.contains_key(".circleci/config.yml"));
            }
        }

        println!("✅ {:?} 平台配置生成成功", platform);
    }

    println!("✅ 多种CI/CD平台支持测试通过");
}

#[tokio::test]
async fn test_cloud_native_manager_integration() {
    println!("🚀 测试云原生管理器集成");

    let k8s_config = KubernetesConfig {
        namespace: "agentx-test".to_string(),
        deployment_name: "agentx-core-test".to_string(),
        service_name: "agentx-service-test".to_string(),
        replicas: 1,
        image: "agentx/core".to_string(),
        image_tag: "test".to_string(),
        resources: ResourceRequirements::default(),
        env_vars: HashMap::new(),
        config_maps: vec![],
        secrets: vec![],
        ingress: None,
    };

    let docker_config = DockerConfig {
        image_name: "agentx/core".to_string(),
        tag: "test".to_string(),
        dockerfile_path: "Dockerfile".to_string(),
        build_context: ".".to_string(),
        build_args: HashMap::new(),
        ports: vec![],
        volumes: vec![],
        environment: HashMap::new(),
    };

    let cloud_provider_config = CloudProviderConfig {
        provider: CloudProvider::AWS,
        region: "us-west-2".to_string(),
        credentials: CloudCredentials {
            access_key: Some("test-key".to_string()),
            secret_key: Some("test-secret".to_string()),
            token: None,
            service_account: None,
        },
        networking: NetworkConfig {
            vpc_id: Some("vpc-12345".to_string()),
            subnet_ids: vec!["subnet-12345".to_string()],
            security_groups: vec!["sg-12345".to_string()],
            load_balancer: None,
        },
        storage: StorageConfig {
            persistent_volumes: vec![],
            object_storage: None,
        },
    };

    let manager = CloudNativeManager::new()
        .with_kubernetes(k8s_config)
        .with_docker(docker_config)
        .with_cloud_provider(cloud_provider_config);

    // 测试部署文件生成
    let deployment_files = manager.generate_deployment_files().await.unwrap();
    assert!(deployment_files.contains_key("deployment.yaml"));
    assert!(deployment_files.contains_key("service.yaml"));
    assert!(deployment_files.contains_key("Dockerfile"));
    assert!(deployment_files.contains_key("docker-compose.yml"));

    // 测试配置验证
    let warnings = manager.validate_configuration().await.unwrap();
    // 可能会有一些配置警告，但不应该有错误
    println!("配置验证警告: {:?}", warnings);

    println!("✅ 云原生管理器集成测试通过");
}
