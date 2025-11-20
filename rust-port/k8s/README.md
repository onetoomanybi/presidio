# Presidio Kubernetes Deployment

This directory contains Kubernetes manifests for deploying Presidio services.

## Prerequisites

- Kubernetes cluster (v1.24+)
- kubectl configured
- (Optional) NGINX Ingress Controller
- (Optional) cert-manager for TLS certificates

## Quick Start

### Deploy Both Services

```bash
# Apply all manifests
kubectl apply -f analyzer-deployment.yaml
kubectl apply -f anonymizer-deployment.yaml

# Verify deployments
kubectl get pods -l app=presidio-analyzer
kubectl get pods -l app=presidio-anonymizer
```

### Expose Services (Optional)

```bash
# Apply ingress (requires NGINX Ingress Controller)
kubectl apply -f ingress.yaml

# Or use port-forward for local access
kubectl port-forward svc/presidio-analyzer 3000:80
kubectl port-forward svc/presidio-anonymizer 3001:80
```

## Components

### Analyzer Service

- **Deployment**: `presidio-analyzer`
  - 3 replicas (configurable)
  - Auto-scaling: 2-10 pods based on CPU/memory
  - Resource requests: 256Mi RAM, 250m CPU
  - Resource limits: 512Mi RAM, 500m CPU

- **Service**: `presidio-analyzer`
  - Type: ClusterIP
  - Port: 80 -> 3000

### Anonymizer Service

- **Deployment**: `presidio-anonymizer`
  - 3 replicas (configurable)
  - Auto-scaling: 2-10 pods based on CPU/memory
  - Resource requests: 256Mi RAM, 250m CPU
  - Resource limits: 512Mi RAM, 500m CPU

- **Service**: `presidio-anonymizer`
  - Type: ClusterIP
  - Port: 80 -> 3001

## Configuration

### Environment Variables

Both services support:

- `RUST_LOG`: Log level (default: `info`)
- `PRESIDIO_ANALYZER_ADDR` / `PRESIDIO_ANONYMIZER_ADDR`: Bind address

### Resource Tuning

Adjust resources in the deployment manifests:

```yaml
resources:
  requests:
    memory: "256Mi"  # Increase for larger workloads
    cpu: "250m"
  limits:
    memory: "512Mi"
    cpu: "500m"
```

### Scaling

Manual scaling:
```bash
kubectl scale deployment presidio-analyzer --replicas=5
```

Auto-scaling is configured via HorizontalPodAutoscaler:
- Min: 2 replicas
- Max: 10 replicas
- Triggers: CPU > 70% or Memory > 80%

## Health Checks

Both services expose a `/health` endpoint:

```bash
# Check analyzer health
curl http://presidio-analyzer/health

# Check anonymizer health
curl http://presidio-anonymizer/health
```

## Security

Security features enabled:
- Non-root user (UID 1000)
- Read-only root filesystem
- No privilege escalation
- Dropped all capabilities
- Network policies (add as needed)

### Adding Network Policies

Create a NetworkPolicy to restrict traffic:

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: presidio-network-policy
spec:
  podSelector:
    matchLabels:
      app: presidio-analyzer
  policyTypes:
  - Ingress
  ingress:
  - from:
    - podSelector:
        matchLabels:
          role: frontend
    ports:
    - protocol: TCP
      port: 3000
```

## Monitoring

### Prometheus Integration

Add Prometheus annotations to services:

```yaml
metadata:
  annotations:
    prometheus.io/scrape: "true"
    prometheus.io/port: "3000"
    prometheus.io/path: "/metrics"
```

### Logging

Logs are sent to stdout/stderr and can be collected by:
- Fluentd
- Filebeat
- CloudWatch Logs (EKS)
- Stackdriver (GKE)

View logs:
```bash
kubectl logs -f deployment/presidio-analyzer
kubectl logs -f deployment/presidio-anonymizer
```

## Troubleshooting

### Pods not starting

```bash
# Check pod status
kubectl describe pod <pod-name>

# Check events
kubectl get events --sort-by='.lastTimestamp'
```

### Service not accessible

```bash
# Check service endpoints
kubectl get endpoints presidio-analyzer

# Test from within cluster
kubectl run -it --rm debug --image=curlimages/curl --restart=Never -- \
  curl http://presidio-analyzer/health
```

### Performance Issues

```bash
# Check resource usage
kubectl top pods -l app=presidio-analyzer

# Check HPA status
kubectl get hpa
```

## Production Recommendations

1. **Use specific image tags** instead of `latest`
   ```yaml
   image: presidio/analyzer:v0.1.0
   ```

2. **Enable Pod Disruption Budgets**
   ```yaml
   apiVersion: policy/v1
   kind: PodDisruptionBudget
   metadata:
     name: presidio-analyzer-pdb
   spec:
     minAvailable: 2
     selector:
       matchLabels:
         app: presidio-analyzer
   ```

3. **Use resource quotas** per namespace

4. **Implement network policies** for zero-trust security

5. **Enable service mesh** (Istio, Linkerd) for advanced traffic management

6. **Set up monitoring and alerting** with Prometheus + Grafana

7. **Configure persistent volumes** if state storage is needed

8. **Use secrets** for sensitive configuration
   ```bash
   kubectl create secret generic presidio-secrets \
     --from-literal=encryption-key=<your-key>
   ```

## Cleanup

```bash
kubectl delete -f .
```
