# AgentX Deployment Guide

## 📖 Overview

This guide provides detailed instructions for deploying AgentX in different environments, including local development, Docker containerization, Kubernetes clusters, and cloud platform deployment.

[English Version](deployment-guide.md) | [中文版本](deployment-guide-cn.md)

## 🏠 Local Development Deployment

### System Requirements

- **Rust**: 1.70+
- **Node.js**: 18+ (for Mastra plugin)
- **Python**: 3.8+ (for LangChain/AutoGen plugins)
- **PostgreSQL**: 13+ (optional, for persistent storage)
- **Redis**: 6+ (optional, for caching)

### Quick Start

```bash
# 1. Clone the project
git clone https://github.com/agentx/agentx.git
cd agentx

# 2. Build the project
cargo build --release

# 3. Configure environment variables
cp .env.example .env
# Edit .env file to set necessary configurations

# 4. Start core service
cargo run --bin agentx-server

# 5. Start HTTP API server
cargo run --example http_server_demo

# 6. Start plugins (in new terminals)
cd plugins/langchain && python langchain_plugin.py &
cd plugins/autogen && python autogen_plugin.py &
cd plugins/mastra && node mastra_plugin.js &
```

### Configuration File

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

## 🐳 Docker Deployment

### Single Container Deployment

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
# Build image
docker build -t agentx:latest .

# Run container
docker run -d \
  --name agentx-core \
  -p 8080:8080 \
  -p 50051:50051 \
  -e RUST_LOG=info \
  -e DATABASE_URL=postgresql://agentx:password@db:5432/agentx \
  agentx:latest
```

### Docker Compose Deployment

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

### Startup and Management

```bash
# Start all services
docker-compose up -d

# Check service status
docker-compose ps

# View logs
docker-compose logs -f agentx-core

# Stop services
docker-compose down

# Update services
docker-compose pull
docker-compose up -d --force-recreate
```

## ☸️ Kubernetes Deployment

### Namespace and Configuration

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

### Core Service Deployment

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

### Ingress Configuration

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

### Deployment Script

```bash
#!/bin/bash
# scripts/deploy-k8s.sh

set -e

echo "Deploying AgentX to Kubernetes..."

# Create namespace
kubectl apply -f k8s/namespace.yaml

# Deploy configurations and secrets
kubectl apply -f k8s/configmap.yaml
kubectl apply -f k8s/secret.yaml

# Deploy database
kubectl apply -f k8s/postgres.yaml
kubectl apply -f k8s/redis.yaml

# Wait for database to be ready
echo "Waiting for database to be ready..."
kubectl wait --for=condition=ready pod -l app=postgres -n agentx --timeout=300s

# Deploy core service
kubectl apply -f k8s/agentx-core.yaml

# Wait for core service to be ready
echo "Waiting for core service to be ready..."
kubectl wait --for=condition=ready pod -l app=agentx-core -n agentx --timeout=300s

# Deploy plugins
kubectl apply -f k8s/plugins.yaml

# Deploy Ingress
kubectl apply -f k8s/ingress.yaml

echo "Deployment completed!"
echo "Checking service status..."
kubectl get pods -n agentx
kubectl get services -n agentx
```

## ☁️ Cloud Platform Deployment

### AWS EKS Deployment

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
# Create EKS cluster
eksctl create cluster -f aws/eks-cluster.yaml

# Configure kubectl
aws eks update-kubeconfig --region us-west-2 --name agentx-cluster

# Deploy AgentX
./scripts/deploy-k8s.sh
```

### Google GKE Deployment

```bash
# Create GKE cluster
gcloud container clusters create agentx-cluster \
  --zone=us-central1-a \
  --num-nodes=3 \
  --machine-type=e2-medium \
  --enable-autoscaling \
  --min-nodes=1 \
  --max-nodes=10

# Get credentials
gcloud container clusters get-credentials agentx-cluster --zone=us-central1-a

# Deploy AgentX
./scripts/deploy-k8s.sh
```

### Azure AKS Deployment

```bash
# Create resource group
az group create --name agentx-rg --location eastus

# Create AKS cluster
az aks create \
  --resource-group agentx-rg \
  --name agentx-cluster \
  --node-count 3 \
  --node-vm-size Standard_B2s \
  --enable-cluster-autoscaler \
  --min-count 1 \
  --max-count 10 \
  --generate-ssh-keys

# Get credentials
az aks get-credentials --resource-group agentx-rg --name agentx-cluster

# Deploy AgentX
./scripts/deploy-k8s.sh
```

## 🔧 Operations Management

### Monitoring and Logging

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

### Backup and Recovery

```bash
#!/bin/bash
# scripts/backup.sh

BACKUP_DIR="/backups/$(date +%Y%m%d_%H%M%S)"
mkdir -p $BACKUP_DIR

# Backup database
kubectl exec -n agentx postgres-0 -- pg_dump -U agentx agentx > $BACKUP_DIR/database.sql

# Backup configurations
kubectl get configmap -n agentx -o yaml > $BACKUP_DIR/configmaps.yaml
kubectl get secret -n agentx -o yaml > $BACKUP_DIR/secrets.yaml

# Compress backup
tar -czf $BACKUP_DIR.tar.gz $BACKUP_DIR
rm -rf $BACKUP_DIR

echo "Backup completed: $BACKUP_DIR.tar.gz"
```

### Scaling and Updates

```bash
# Horizontal scaling
kubectl scale deployment agentx-core --replicas=5 -n agentx

# Rolling update
kubectl set image deployment/agentx-core agentx-core=agentx/core:v1.1.0 -n agentx

# Check update status
kubectl rollout status deployment/agentx-core -n agentx

# Rollback update
kubectl rollout undo deployment/agentx-core -n agentx
```

This deployment guide provides complete deployment solutions for AgentX in various environments, ensuring high availability and scalability of the system.
