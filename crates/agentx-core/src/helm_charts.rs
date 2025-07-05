//! Helm Charts生成器
//! 
//! 提供AgentX项目的Helm Charts自动生成功能

use crate::cloud_native::ResourceRequirements;
use agentx_a2a::A2AResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

/// Helm Chart配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelmChartConfig {
    /// Chart名称
    pub name: String,
    /// Chart版本
    pub version: String,
    /// 应用版本
    pub app_version: String,
    /// 描述
    pub description: String,
    /// 关键词
    pub keywords: Vec<String>,
    /// 维护者
    pub maintainers: Vec<Maintainer>,
    /// 依赖
    pub dependencies: Vec<Dependency>,
    /// 默认值
    pub default_values: HelmValues,
}

/// 维护者信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Maintainer {
    /// 姓名
    pub name: String,
    /// 邮箱
    pub email: String,
    /// 网站
    pub url: Option<String>,
}

/// Chart依赖
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    /// 依赖名称
    pub name: String,
    /// 版本
    pub version: String,
    /// 仓库
    pub repository: String,
    /// 条件
    pub condition: Option<String>,
    /// 标签
    pub tags: Vec<String>,
}

/// Helm Values配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelmValues {
    /// 镜像配置
    pub image: ImageConfig,
    /// 副本数
    pub replica_count: u32,
    /// 服务配置
    pub service: ServiceConfig,
    /// Ingress配置
    pub ingress: IngressValues,
    /// 资源配置
    pub resources: ResourceRequirements,
    /// 自动扩缩容
    pub autoscaling: AutoscalingConfig,
    /// 节点选择器
    pub node_selector: HashMap<String, String>,
    /// 容忍度
    pub tolerations: Vec<Toleration>,
    /// 亲和性
    pub affinity: Option<String>,
    /// 环境变量
    pub env: HashMap<String, String>,
    /// 配置映射
    pub config_maps: Vec<String>,
    /// 密钥
    pub secrets: Vec<String>,
}

/// 镜像配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageConfig {
    /// 仓库
    pub repository: String,
    /// 拉取策略
    pub pull_policy: String,
    /// 标签
    pub tag: String,
    /// 镜像拉取密钥
    pub pull_secrets: Vec<String>,
}

/// 服务配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    /// 服务类型
    pub service_type: String,
    /// 端口
    pub port: u16,
    /// 目标端口
    pub target_port: u16,
    /// 节点端口
    pub node_port: Option<u16>,
    /// 注解
    pub annotations: HashMap<String, String>,
}

/// Ingress Values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngressValues {
    /// 是否启用
    pub enabled: bool,
    /// 类名
    pub class_name: Option<String>,
    /// 注解
    pub annotations: HashMap<String, String>,
    /// 主机
    pub hosts: Vec<IngressHost>,
    /// TLS配置
    pub tls: Vec<IngressTLS>,
}

/// Ingress主机配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngressHost {
    /// 主机名
    pub host: String,
    /// 路径
    pub paths: Vec<IngressPath>,
}

/// Ingress路径配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngressPath {
    /// 路径
    pub path: String,
    /// 路径类型
    pub path_type: String,
}

/// Ingress TLS配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngressTLS {
    /// 密钥名称
    pub secret_name: String,
    /// 主机列表
    pub hosts: Vec<String>,
}

/// 自动扩缩容配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoscalingConfig {
    /// 是否启用
    pub enabled: bool,
    /// 最小副本数
    pub min_replicas: u32,
    /// 最大副本数
    pub max_replicas: u32,
    /// 目标CPU使用率
    pub target_cpu_utilization: u32,
    /// 目标内存使用率
    pub target_memory_utilization: Option<u32>,
}

/// 容忍度配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Toleration {
    /// 键
    pub key: String,
    /// 操作符
    pub operator: String,
    /// 值
    pub value: Option<String>,
    /// 效果
    pub effect: String,
}

/// Helm Charts生成器
pub struct HelmChartsGenerator {
    /// Chart配置
    config: HelmChartConfig,
}

impl HelmChartsGenerator {
    /// 创建新的Helm Charts生成器
    pub fn new(config: HelmChartConfig) -> Self {
        info!("📦 创建Helm Charts生成器: {}", config.name);
        Self { config }
    }

    /// 生成Chart.yaml
    pub fn generate_chart_yaml(&self) -> A2AResult<String> {
        debug!("生成Chart.yaml");
        
        let maintainers = self.config.maintainers.iter()
            .map(|m| format!("  - name: {}\n    email: {}", m.name, m.email))
            .collect::<Vec<_>>()
            .join("\n");

        let dependencies = if !self.config.dependencies.is_empty() {
            let deps = self.config.dependencies.iter()
                .map(|d| format!("  - name: {}\n    version: {}\n    repository: {}", d.name, d.version, d.repository))
                .collect::<Vec<_>>()
                .join("\n");
            format!("dependencies:\n{}", deps)
        } else {
            String::new()
        };

        let chart_yaml = format!(r#"apiVersion: v2
name: {}
description: {}
type: application
version: {}
appVersion: "{}"
keywords:
{}
maintainers:
{}
{}
"#,
            self.config.name,
            self.config.description,
            self.config.version,
            self.config.app_version,
            self.config.keywords.iter().map(|k| format!("  - {}", k)).collect::<Vec<_>>().join("\n"),
            maintainers,
            dependencies
        );

        Ok(chart_yaml)
    }

    /// 生成values.yaml
    pub fn generate_values_yaml(&self) -> A2AResult<String> {
        debug!("生成values.yaml");
        
        let values = &self.config.default_values;
        
        let env_vars = values.env.iter()
            .map(|(k, v)| format!("  {}: \"{}\"", k, v))
            .collect::<Vec<_>>()
            .join("\n");

        let ingress_hosts = values.ingress.hosts.iter()
            .map(|h| {
                let paths = h.paths.iter()
                    .map(|p| format!("      - path: {}\n        pathType: {}", p.path, p.path_type))
                    .collect::<Vec<_>>()
                    .join("\n");
                format!("  - host: {}\n    http:\n      paths:\n{}", h.host, paths)
            })
            .collect::<Vec<_>>()
            .join("\n");

        let values_yaml = format!(r#"# AgentX Helm Chart默认配置

replicaCount: {}

image:
  repository: {}
  pullPolicy: {}
  tag: "{}"

imagePullSecrets: []

nameOverride: ""
fullnameOverride: ""

serviceAccount:
  create: true
  annotations: {{}}
  name: ""

podAnnotations: {{}}

podSecurityContext: {{}}

securityContext: {{}}

service:
  type: {}
  port: {}
  targetPort: {}

ingress:
  enabled: {}
  className: "{}"
  annotations: {{}}
  hosts:
{}
  tls: []

resources:
  limits:
    cpu: {}
    memory: {}
  requests:
    cpu: {}
    memory: {}

autoscaling:
  enabled: {}
  minReplicas: {}
  maxReplicas: {}
  targetCPUUtilizationPercentage: {}

nodeSelector: {{}}

tolerations: []

affinity: {{}}

env:
{}

configMaps: {}

secrets: {}
"#,
            values.replica_count,
            values.image.repository,
            values.image.pull_policy,
            values.image.tag,
            values.service.service_type,
            values.service.port,
            values.service.target_port,
            values.ingress.enabled,
            values.ingress.class_name.as_deref().unwrap_or(""),
            ingress_hosts,
            values.resources.cpu_limit,
            values.resources.memory_limit,
            values.resources.cpu_request,
            values.resources.memory_request,
            values.autoscaling.enabled,
            values.autoscaling.min_replicas,
            values.autoscaling.max_replicas,
            values.autoscaling.target_cpu_utilization,
            env_vars,
            format!("{:?}", values.config_maps),
            format!("{:?}", values.secrets)
        );

        Ok(values_yaml)
    }

    /// 生成deployment.yaml模板
    pub fn generate_deployment_template(&self) -> A2AResult<String> {
        debug!("生成deployment.yaml模板");
        
        let template = r#"apiVersion: apps/v1
kind: Deployment
metadata:
  name: {{ include "agentx.fullname" . }}
  labels:
    {{- include "agentx.labels" . | nindent 4 }}
spec:
  {{- if not .Values.autoscaling.enabled }}
  replicas: {{ .Values.replicaCount }}
  {{- end }}
  selector:
    matchLabels:
      {{- include "agentx.selectorLabels" . | nindent 6 }}
  template:
    metadata:
      {{- with .Values.podAnnotations }}
      annotations:
        {{- toYaml . | nindent 8 }}
      {{- end }}
      labels:
        {{- include "agentx.selectorLabels" . | nindent 8 }}
    spec:
      {{- with .Values.imagePullSecrets }}
      imagePullSecrets:
        {{- toYaml . | nindent 8 }}
      {{- end }}
      serviceAccountName: {{ include "agentx.serviceAccountName" . }}
      securityContext:
        {{- toYaml .Values.podSecurityContext | nindent 8 }}
      containers:
        - name: {{ .Chart.Name }}
          securityContext:
            {{- toYaml .Values.securityContext | nindent 12 }}
          image: "{{ .Values.image.repository }}:{{ .Values.image.tag | default .Chart.AppVersion }}"
          imagePullPolicy: {{ .Values.image.pullPolicy }}
          ports:
            - name: grpc
              containerPort: 50051
              protocol: TCP
            - name: http
              containerPort: 8080
              protocol: TCP
          env:
            {{- range $key, $value := .Values.env }}
            - name: {{ $key }}
              value: {{ $value | quote }}
            {{- end }}
          livenessProbe:
            httpGet:
              path: /health
              port: http
            initialDelaySeconds: 30
            periodSeconds: 10
          readinessProbe:
            httpGet:
              path: /ready
              port: http
            initialDelaySeconds: 5
            periodSeconds: 5
          resources:
            {{- toYaml .Values.resources | nindent 12 }}
      {{- with .Values.nodeSelector }}
      nodeSelector:
        {{- toYaml . | nindent 8 }}
      {{- end }}
      {{- with .Values.affinity }}
      affinity:
        {{- toYaml . | nindent 8 }}
      {{- end }}
      {{- with .Values.tolerations }}
      tolerations:
        {{- toYaml . | nindent 8 }}
      {{- end }}
"#;

        Ok(template.to_string())
    }

    /// 生成service.yaml模板
    pub fn generate_service_template(&self) -> A2AResult<String> {
        debug!("生成service.yaml模板");
        
        let template = r#"apiVersion: v1
kind: Service
metadata:
  name: {{ include "agentx.fullname" . }}
  labels:
    {{- include "agentx.labels" . | nindent 4 }}
spec:
  type: {{ .Values.service.type }}
  ports:
    - port: {{ .Values.service.port }}
      targetPort: http
      protocol: TCP
      name: http
    - port: 50051
      targetPort: grpc
      protocol: TCP
      name: grpc
  selector:
    {{- include "agentx.selectorLabels" . | nindent 4 }}
"#;

        Ok(template.to_string())
    }

    /// 生成ingress.yaml模板
    pub fn generate_ingress_template(&self) -> A2AResult<String> {
        debug!("生成ingress.yaml模板");
        
        let template = r#"{{- if .Values.ingress.enabled -}}
{{- $fullName := include "agentx.fullname" . -}}
{{- $svcPort := .Values.service.port -}}
{{- if and .Values.ingress.className (not (hasKey .Values.ingress.annotations "kubernetes.io/ingress.class")) }}
  {{- $_ := set .Values.ingress.annotations "kubernetes.io/ingress.class" .Values.ingress.className}}
{{- end }}
{{- if semverCompare ">=1.19-0" .Capabilities.KubeVersion.GitVersion -}}
apiVersion: networking.k8s.io/v1
{{- else if semverCompare ">=1.14-0" .Capabilities.KubeVersion.GitVersion -}}
apiVersion: networking.k8s.io/v1beta1
{{- else -}}
apiVersion: extensions/v1beta1
{{- end }}
kind: Ingress
metadata:
  name: {{ $fullName }}
  labels:
    {{- include "agentx.labels" . | nindent 4 }}
  {{- with .Values.ingress.annotations }}
  annotations:
    {{- toYaml . | nindent 4 }}
  {{- end }}
spec:
  {{- if and .Values.ingress.className (semverCompare ">=1.18-0" .Capabilities.KubeVersion.GitVersion) }}
  ingressClassName: {{ .Values.ingress.className }}
  {{- end }}
  {{- if .Values.ingress.tls }}
  tls:
    {{- range .Values.ingress.tls }}
    - hosts:
        {{- range .hosts }}
        - {{ . | quote }}
        {{- end }}
      secretName: {{ .secretName }}
    {{- end }}
  {{- end }}
  rules:
    {{- range .Values.ingress.hosts }}
    - host: {{ .host | quote }}
      http:
        paths:
          {{- range .paths }}
          - path: {{ .path }}
            {{- if and .pathType (semverCompare ">=1.18-0" $.Capabilities.KubeVersion.GitVersion) }}
            pathType: {{ .pathType }}
            {{- end }}
            backend:
              {{- if semverCompare ">=1.19-0" $.Capabilities.KubeVersion.GitVersion }}
              service:
                name: {{ $fullName }}
                port:
                  number: {{ $svcPort }}
              {{- else }}
              serviceName: {{ $fullName }}
              servicePort: {{ $svcPort }}
              {{- end }}
          {{- end }}
    {{- end }}
{{- end }}
"#;

        Ok(template.to_string())
    }

    /// 生成_helpers.tpl模板
    pub fn generate_helpers_template(&self) -> A2AResult<String> {
        debug!("生成_helpers.tpl模板");
        
        let template = r#"{{/*
Expand the name of the chart.
*/}}
{{- define "agentx.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
*/}}
{{- define "agentx.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.nameOverride }}
{{- if contains $name .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}
{{- end }}

{{/*
Create chart name and version as used by the chart label.
*/}}
{{- define "agentx.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "agentx.labels" -}}
helm.sh/chart: {{ include "agentx.chart" . }}
{{ include "agentx.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{/*
Selector labels
*/}}
{{- define "agentx.selectorLabels" -}}
app.kubernetes.io/name: {{ include "agentx.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Create the name of the service account to use
*/}}
{{- define "agentx.serviceAccountName" -}}
{{- if .Values.serviceAccount.create }}
{{- default (include "agentx.fullname" .) .Values.serviceAccount.name }}
{{- else }}
{{- default "default" .Values.serviceAccount.name }}
{{- end }}
{{- end }}
"#;

        Ok(template.to_string())
    }

    /// 生成所有Helm Chart文件
    pub fn generate_all_files(&self) -> A2AResult<HashMap<String, String>> {
        info!("生成所有Helm Chart文件");
        
        let mut files = HashMap::new();
        
        files.insert("Chart.yaml".to_string(), self.generate_chart_yaml()?);
        files.insert("values.yaml".to_string(), self.generate_values_yaml()?);
        files.insert("templates/deployment.yaml".to_string(), self.generate_deployment_template()?);
        files.insert("templates/service.yaml".to_string(), self.generate_service_template()?);
        files.insert("templates/ingress.yaml".to_string(), self.generate_ingress_template()?);
        files.insert("templates/_helpers.tpl".to_string(), self.generate_helpers_template()?);
        
        // 生成HPA模板
        files.insert("templates/hpa.yaml".to_string(), self.generate_hpa_template()?);
        
        // 生成ServiceAccount模板
        files.insert("templates/serviceaccount.yaml".to_string(), self.generate_serviceaccount_template()?);
        
        info!("生成了 {} 个Helm Chart文件", files.len());
        Ok(files)
    }

    /// 生成HPA模板
    fn generate_hpa_template(&self) -> A2AResult<String> {
        let template = r#"{{- if .Values.autoscaling.enabled }}
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: {{ include "agentx.fullname" . }}
  labels:
    {{- include "agentx.labels" . | nindent 4 }}
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: {{ include "agentx.fullname" . }}
  minReplicas: {{ .Values.autoscaling.minReplicas }}
  maxReplicas: {{ .Values.autoscaling.maxReplicas }}
  metrics:
    {{- if .Values.autoscaling.targetCPUUtilizationPercentage }}
    - type: Resource
      resource:
        name: cpu
        target:
          type: Utilization
          averageUtilization: {{ .Values.autoscaling.targetCPUUtilizationPercentage }}
    {{- end }}
    {{- if .Values.autoscaling.targetMemoryUtilizationPercentage }}
    - type: Resource
      resource:
        name: memory
        target:
          type: Utilization
          averageUtilization: {{ .Values.autoscaling.targetMemoryUtilizationPercentage }}
    {{- end }}
{{- end }}
"#;
        Ok(template.to_string())
    }

    /// 生成ServiceAccount模板
    fn generate_serviceaccount_template(&self) -> A2AResult<String> {
        let template = r#"{{- if .Values.serviceAccount.create -}}
apiVersion: v1
kind: ServiceAccount
metadata:
  name: {{ include "agentx.serviceAccountName" . }}
  labels:
    {{- include "agentx.labels" . | nindent 4 }}
  {{- with .Values.serviceAccount.annotations }}
  annotations:
    {{- toYaml . | nindent 4 }}
  {{- end }}
{{- end }}
"#;
        Ok(template.to_string())
    }
}

impl Default for HelmChartConfig {
    fn default() -> Self {
        Self {
            name: "agentx".to_string(),
            version: "0.1.0".to_string(),
            app_version: "0.1.0".to_string(),
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
            dependencies: vec![],
            default_values: HelmValues::default(),
        }
    }
}

impl Default for HelmValues {
    fn default() -> Self {
        Self {
            image: ImageConfig {
                repository: "agentx/core".to_string(),
                pull_policy: "IfNotPresent".to_string(),
                tag: "latest".to_string(),
                pull_secrets: vec![],
            },
            replica_count: 1,
            service: ServiceConfig {
                service_type: "ClusterIP".to_string(),
                port: 8080,
                target_port: 8080,
                node_port: None,
                annotations: HashMap::new(),
            },
            ingress: IngressValues {
                enabled: false,
                class_name: Some("nginx".to_string()),
                annotations: HashMap::new(),
                hosts: vec![],
                tls: vec![],
            },
            resources: ResourceRequirements {
                cpu_request: "100m".to_string(),
                cpu_limit: "500m".to_string(),
                memory_request: "128Mi".to_string(),
                memory_limit: "512Mi".to_string(),
            },
            autoscaling: AutoscalingConfig {
                enabled: false,
                min_replicas: 1,
                max_replicas: 10,
                target_cpu_utilization: 80,
                target_memory_utilization: None,
            },
            node_selector: HashMap::new(),
            tolerations: vec![],
            affinity: None,
            env: HashMap::new(),
            config_maps: vec![],
            secrets: vec![],
        }
    }
}
