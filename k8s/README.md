# AgentMem Kubernetes Deployment Guide

## Overview

This directory contains Kubernetes manifests for deploying AgentMem in production.

## Prerequisites

- Kubernetes 1.24+
- kubectl configured with cluster access
- Helm 3.x (optional, for Helm installation)

## Quick Start

### 1. Create Namespace

```bash
kubectl apply -f agentmem-deployment.yaml -f agentmem-config.yaml
```

### 2. Update Secrets

Edit `agentmem-config.yaml` and update the secret values:

```bash
# Encode secrets (base64)
echo -n "your-jwt-secret" | base64

# Update the secret
kubectl edit secret agentmem-secrets -n agentmem
```

### 3. Deploy

```bash
kubectl apply -f agentmem-deployment.yaml
kubectl apply -f agentmem-config.yaml
```

### 4. Verify Deployment

```bash
# Check pods
kubectl get pods -n agentmem

# Check services
kubectl get svc -n agentmem

# Check logs
kubectl logs -n agentmem -l app=agentmem
```

## Scaling

### Manual Scaling

```bash
kubectl scale deployment agentmem-server -n agentmem --replicas=5
```

### Horizontal Pod Autoscaling

The HPA is configured in `agentmem-deployment.yaml`:
- Min replicas: 2
- Max replicas: 10
- CPU target: 70%
- Memory target: 80%

### Enable HPA

```bash
kubectl autoscale deployment agentmem-server -n agentmem --min=2 --max=10 --cpu-percent=70
```

## Monitoring

### Prometheus Metrics

Access metrics at: `http://<service>:8080/metrics`

### Health Check

```bash
kubectl exec -it <pod-name> -n agentmem -- curl http://localhost:8080/health
```

## Upgrading

### Rolling Update

```bash
# Update image version in deployment
kubectl set image deployment/agentmem-server agentmem=agentmem/server:v2.0.0 -n agentmem

# Check rollout status
kubectl rollout status deployment/agentmem-server -n agentmem
```

### Rollback

```bash
kubectl rollout undo deployment/agentmem-server -n agentmem
```

## Troubleshooting

### Pod Not Starting

```bash
# Check pod status
kubectl describe pod <pod-name> -n agentmem

# Check events
kubectl get events -n agentmem --sort-by='.lastTimestamp'
```

### High Memory Usage

```bash
# Check resource usage
kubectl top pods -n agentmem

# Increase limits in deployment
kubectl edit deployment agentmem-server -n agentmem
```

### Connection Issues

```bash
# Check service endpoints
kubectl get endpoints agentmem-service -n agentmem

# Test service connectivity
kubectl run test --rm -it --image=busybox -- wget -O- http://agentmem-service/health
```

## Production Checklist

- [ ] Update all secrets with real values
- [ ] Configure TLS certificates
- [ ] Set up Prometheus/Grafana monitoring
- [ ] Configure resource limits based on load testing
- [ ] Set up backup strategy for PVC
- [ ] Configure network policies
- [ ] Enable audit logging

## Helm Installation (Alternative)

```bash
# Add Helm repo (if published)
helm repo add agentmem https://charts.agentmem.io
helm repo update

# Install
helm install agentmem agentmem/agentmem -n agentmem --create-namespace
```

## Resources

- [AgentMem Documentation](https://github.com/louloulin/AgentMem)
- [Kubernetes Documentation](https://kubernetes.io/docs/)
- [Production Best Practices](https://kubernetes.io/docs/setup/production-environment/)
