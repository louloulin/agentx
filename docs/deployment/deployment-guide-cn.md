# AgentX 部署指南

## 📖 概述

本指南详细介绍AgentX在不同环境中的部署方案，包括本地开发、Docker容器化、Kubernetes集群和云平台部署。

[English Version](deployment-guide.md) | [中文版本](deployment-guide-cn.md)

## 🏠 本地开发部署

### 环境要求

- **Rust**: 1.70+
- **Node.js**: 18+ (Mastra插件)
- **Python**: 3.8+ (LangChain/AutoGen插件)
- **PostgreSQL**: 13+ (可选，用于持久化存储)
- **Redis**: 6+ (可选，用于缓存)

### 快速启动

```bash
# 1. 克隆项目
git clone https://github.com/agentx/agentx.git
cd agentx

# 2. 构建项目
cargo build --release

# 3. 配置环境变量
cp .env.example .env
# 编辑.env文件，设置必要的配置

# 4. 启动核心服务
cargo run --bin agentx-server

# 5. 启动HTTP API服务器
cargo run --example http_server_demo

# 6. 启动插件（新终端）
cd plugins/langchain && python langchain_plugin.py &
cd plugins/autogen && python autogen_plugin.py &
cd plugins/mastra && node mastra_plugin.js &
```

### 配置文件

```yaml
# config/agentx.yaml
server:
  host: "0.0.0.0"
  port: 8080
  grpc_port: 50051

database:
  url: "postgresql://agentx:password@localhost:5432/agentx"
  max_connections: 10

redis:
  url: "redis://localhost:6379"
  pool_size: 10

logging:
  level: "info"
  format: "json"

plugins:
  auto_discovery: true
  timeout: 30s
  health_check_interval: 10s

security:
  enable_tls: false
  jwt_secret: "your-jwt-secret-key"
  api_key_required: false
```

## 🐳 Docker部署

### 单容器部署

```dockerfile
# Dockerfile
FROM rust:1.70 as builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/agentx-server .
COPY --from=builder /app/config ./config

EXPOSE 8080 50051

CMD ["./agentx-server"]
```

```bash
# 构建镜像
docker build -t agentx:latest .

# 运行容器
docker run -d \
  --name agentx-core \
  -p 8080:8080 \
  -p 50051:50051 \
  -e RUST_LOG=info \
  -e DATABASE_URL=postgresql://agentx:password@db:5432/agentx \
  agentx:latest
```

### Docker Compose部署

```yaml
# docker-compose.yml
version: '3.8'

services:
  agentx-core:
    build: .
    ports:
      - "8080:8080"
      - "50051:50051"
    environment:
      - RUST_LOG=info
      - DATABASE_URL=postgresql://agentx:password@postgres:5432/agentx
      - REDIS_URL=redis://redis:6379
    depends_on:
      - postgres
      - redis
    networks:
      - agentx-network
    restart: unless-stopped

  postgres:
    image: postgres:13
    environment:
      - POSTGRES_DB=agentx
      - POSTGRES_USER=agentx
      - POSTGRES_PASSWORD=password
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./scripts/init.sql:/docker-entrypoint-initdb.d/init.sql
    networks:
      - agentx-network
    restart: unless-stopped

  redis:
    image: redis:6-alpine
    command: redis-server --appendonly yes
    volumes:
      - redis_data:/data
    networks:
      - agentx-network
    restart: unless-stopped

  langchain-plugin:
    build: ./plugins/langchain
    ports:
      - "50052:50052"
    environment:
      - OPENAI_API_KEY=${OPENAI_API_KEY}
      - AGENTX_CORE_URL=agentx-core:50051
    depends_on:
      - agentx-core
    networks:
      - agentx-network
    restart: unless-stopped

  autogen-plugin:
    build: ./plugins/autogen
    ports:
      - "50053:50053"
    environment:
      - OPENAI_API_KEY=${OPENAI_API_KEY}
      - AGENTX_CORE_URL=agentx-core:50051
    depends_on:
      - agentx-core
    networks:
      - agentx-network
    restart: unless-stopped

  mastra-plugin:
    build: ./plugins/mastra
    ports:
      - "50054:50054"
    environment:
      - AGENTX_CORE_URL=agentx-core:50051
    depends_on:
      - agentx-core
    networks:
      - agentx-network
    restart: unless-stopped

  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx/nginx.conf:/etc/nginx/nginx.conf
      - ./nginx/ssl:/etc/nginx/ssl
    depends_on:
      - agentx-core
    networks:
      - agentx-network
    restart: unless-stopped

volumes:
  postgres_data:
  redis_data:

networks:
  agentx-network:
    driver: bridge
```

### 启动和管理

```bash
# 启动所有服务
docker-compose up -d

# 查看服务状态
docker-compose ps

# 查看日志
docker-compose logs -f agentx-core

# 停止服务
docker-compose down

# 更新服务
docker-compose pull
docker-compose up -d --force-recreate
```

## ☸️ Kubernetes部署

### 命名空间和配置

```yaml
# k8s/namespace.yaml
apiVersion: v1
kind: Namespace
metadata:
  name: agentx
---
# k8s/configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: agentx-config
  namespace: agentx
data:
  agentx.yaml: |
    server:
      host: "0.0.0.0"
      port: 8080
      grpc_port: 50051
    database:
      url: "postgresql://agentx:password@postgres:5432/agentx"
    redis:
      url: "redis://redis:6379"
    logging:
      level: "info"
---
# k8s/secret.yaml
apiVersion: v1
kind: Secret
metadata:
  name: agentx-secrets
  namespace: agentx
type: Opaque
data:
  database-password: cGFzc3dvcmQ=  # base64 encoded "password"
  jwt-secret: eW91ci1qd3Qtc2VjcmV0LWtleQ==  # base64 encoded "your-jwt-secret-key"
  openai-api-key: c2stWW91ck9wZW5BSUFQSUtleQ==  # base64 encoded API key
```

### 核心服务部署

```yaml
# k8s/agentx-core.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: agentx-core
  namespace: agentx
spec:
  replicas: 3
  selector:
    matchLabels:
      app: agentx-core
  template:
    metadata:
      labels:
        app: agentx-core
    spec:
      containers:
      - name: agentx-core
        image: agentx/core:latest
        ports:
        - containerPort: 8080
          name: http
        - containerPort: 50051
          name: grpc
        env:
        - name: RUST_LOG
          value: "info"
        - name: DATABASE_URL
          value: "postgresql://agentx:$(DATABASE_PASSWORD)@postgres:5432/agentx"
        - name: DATABASE_PASSWORD
          valueFrom:
            secretKeyRef:
              name: agentx-secrets
              key: database-password
        - name: REDIS_URL
          value: "redis://redis:6379"
        volumeMounts:
        - name: config
          mountPath: /app/config
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
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
      volumes:
      - name: config
        configMap:
          name: agentx-config
---
apiVersion: v1
kind: Service
metadata:
  name: agentx-core
  namespace: agentx
spec:
  selector:
    app: agentx-core
  ports:
  - name: http
    port: 8080
    targetPort: 8080
  - name: grpc
    port: 50051
    targetPort: 50051
  type: ClusterIP
```

### 数据库部署

```yaml
# k8s/postgres.yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: postgres
  namespace: agentx
spec:
  serviceName: postgres
  replicas: 1
  selector:
    matchLabels:
      app: postgres
  template:
    metadata:
      labels:
        app: postgres
    spec:
      containers:
      - name: postgres
        image: postgres:13
        env:
        - name: POSTGRES_DB
          value: agentx
        - name: POSTGRES_USER
          value: agentx
        - name: POSTGRES_PASSWORD
          valueFrom:
            secretKeyRef:
              name: agentx-secrets
              key: database-password
        ports:
        - containerPort: 5432
        volumeMounts:
        - name: postgres-storage
          mountPath: /var/lib/postgresql/data
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "1Gi"
            cpu: "500m"
  volumeClaimTemplates:
  - metadata:
      name: postgres-storage
    spec:
      accessModes: ["ReadWriteOnce"]
      resources:
        requests:
          storage: 10Gi
---
apiVersion: v1
kind: Service
metadata:
  name: postgres
  namespace: agentx
spec:
  selector:
    app: postgres
  ports:
  - port: 5432
    targetPort: 5432
  type: ClusterIP
```

### 插件部署

```yaml
# k8s/plugins.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: langchain-plugin
  namespace: agentx
spec:
  replicas: 2
  selector:
    matchLabels:
      app: langchain-plugin
  template:
    metadata:
      labels:
        app: langchain-plugin
    spec:
      containers:
      - name: langchain-plugin
        image: agentx/langchain-plugin:latest
        ports:
        - containerPort: 50052
        env:
        - name: OPENAI_API_KEY
          valueFrom:
            secretKeyRef:
              name: agentx-secrets
              key: openai-api-key
        - name: AGENTX_CORE_URL
          value: "agentx-core:50051"
        resources:
          requests:
            memory: "512Mi"
            cpu: "250m"
          limits:
            memory: "1Gi"
            cpu: "500m"
---
apiVersion: v1
kind: Service
metadata:
  name: langchain-plugin
  namespace: agentx
spec:
  selector:
    app: langchain-plugin
  ports:
  - port: 50052
    targetPort: 50052
  type: ClusterIP
```

### Ingress配置

```yaml
# k8s/ingress.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: agentx-ingress
  namespace: agentx
  annotations:
    kubernetes.io/ingress.class: nginx
    cert-manager.io/cluster-issuer: letsencrypt-prod
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
    nginx.ingress.kubernetes.io/grpc-backend: "true"
spec:
  tls:
  - hosts:
    - api.agentx.example.com
    secretName: agentx-tls
  rules:
  - host: api.agentx.example.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: agentx-core
            port:
              number: 8080
      - path: /grpc
        pathType: Prefix
        backend:
          service:
            name: agentx-core
            port:
              number: 50051
```

### 部署脚本

```bash
#!/bin/bash
# scripts/deploy-k8s.sh

set -e

echo "Deploying AgentX to Kubernetes..."

# 创建命名空间
kubectl apply -f k8s/namespace.yaml

# 部署配置和密钥
kubectl apply -f k8s/configmap.yaml
kubectl apply -f k8s/secret.yaml

# 部署数据库
kubectl apply -f k8s/postgres.yaml
kubectl apply -f k8s/redis.yaml

# 等待数据库就绪
echo "Waiting for database to be ready..."
kubectl wait --for=condition=ready pod -l app=postgres -n agentx --timeout=300s

# 部署核心服务
kubectl apply -f k8s/agentx-core.yaml

# 等待核心服务就绪
echo "Waiting for core service to be ready..."
kubectl wait --for=condition=ready pod -l app=agentx-core -n agentx --timeout=300s

# 部署插件
kubectl apply -f k8s/plugins.yaml

# 部署Ingress
kubectl apply -f k8s/ingress.yaml

echo "Deployment completed!"
echo "Checking service status..."
kubectl get pods -n agentx
kubectl get services -n agentx
```

## ☁️ 云平台部署

### AWS EKS部署

```yaml
# aws/eks-cluster.yaml
apiVersion: eksctl.io/v1alpha5
kind: ClusterConfig

metadata:
  name: agentx-cluster
  region: us-west-2

nodeGroups:
  - name: agentx-workers
    instanceType: t3.medium
    desiredCapacity: 3
    minSize: 1
    maxSize: 10
    volumeSize: 20
    ssh:
      allow: true

addons:
  - name: vpc-cni
  - name: coredns
  - name: kube-proxy
  - name: aws-ebs-csi-driver
```

```bash
# 创建EKS集群
eksctl create cluster -f aws/eks-cluster.yaml

# 配置kubectl
aws eks update-kubeconfig --region us-west-2 --name agentx-cluster

# 部署AgentX
./scripts/deploy-k8s.sh
```

### Google GKE部署

```bash
# 创建GKE集群
gcloud container clusters create agentx-cluster \
  --zone=us-central1-a \
  --num-nodes=3 \
  --machine-type=e2-medium \
  --enable-autoscaling \
  --min-nodes=1 \
  --max-nodes=10

# 获取凭据
gcloud container clusters get-credentials agentx-cluster --zone=us-central1-a

# 部署AgentX
./scripts/deploy-k8s.sh
```

### Azure AKS部署

```bash
# 创建资源组
az group create --name agentx-rg --location eastus

# 创建AKS集群
az aks create \
  --resource-group agentx-rg \
  --name agentx-cluster \
  --node-count 3 \
  --node-vm-size Standard_B2s \
  --enable-cluster-autoscaler \
  --min-count 1 \
  --max-count 10 \
  --generate-ssh-keys

# 获取凭据
az aks get-credentials --resource-group agentx-rg --name agentx-cluster

# 部署AgentX
./scripts/deploy-k8s.sh
```

## 🔧 运维管理

### 监控和日志

```yaml
# k8s/monitoring.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: prometheus-config
  namespace: agentx
data:
  prometheus.yml: |
    global:
      scrape_interval: 15s
    scrape_configs:
    - job_name: 'agentx-core'
      static_configs:
      - targets: ['agentx-core:8080']
      metrics_path: /metrics
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: prometheus
  namespace: agentx
spec:
  replicas: 1
  selector:
    matchLabels:
      app: prometheus
  template:
    metadata:
      labels:
        app: prometheus
    spec:
      containers:
      - name: prometheus
        image: prom/prometheus:latest
        ports:
        - containerPort: 9090
        volumeMounts:
        - name: config
          mountPath: /etc/prometheus
      volumes:
      - name: config
        configMap:
          name: prometheus-config
```

### 备份和恢复

```bash
#!/bin/bash
# scripts/backup.sh

BACKUP_DIR="/backups/$(date +%Y%m%d_%H%M%S)"
mkdir -p $BACKUP_DIR

# 备份数据库
kubectl exec -n agentx postgres-0 -- pg_dump -U agentx agentx > $BACKUP_DIR/database.sql

# 备份配置
kubectl get configmap -n agentx -o yaml > $BACKUP_DIR/configmaps.yaml
kubectl get secret -n agentx -o yaml > $BACKUP_DIR/secrets.yaml

# 压缩备份
tar -czf $BACKUP_DIR.tar.gz $BACKUP_DIR
rm -rf $BACKUP_DIR

echo "Backup completed: $BACKUP_DIR.tar.gz"
```

### 扩容和更新

```bash
# 水平扩容
kubectl scale deployment agentx-core --replicas=5 -n agentx

# 滚动更新
kubectl set image deployment/agentx-core agentx-core=agentx/core:v1.1.0 -n agentx

# 查看更新状态
kubectl rollout status deployment/agentx-core -n agentx

# 回滚更新
kubectl rollout undo deployment/agentx-core -n agentx
```

这个部署指南提供了AgentX在各种环境中的完整部署方案，确保系统的高可用性和可扩展性。
