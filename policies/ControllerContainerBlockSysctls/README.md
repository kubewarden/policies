# Container Block Sysctls

Setting sysctls can allow containers unauthorized escalated privileges to a Kubernetes node.

You should not set `securityContext.sysctls`

```
...
  spec:
    securityContext:
      sysctls
```

https://kubernetes.io/docs/tasks/configure-pod-container/security-context/

# Settings

```yaml
settings:
  exclude_namespaces: [] # optional, default: ["kube-system"]
  exclude_label_key: "" # optional
  exclude_label_value: "" # optional
```

# Resources

Policy applies to resources kinds:
`Deployment`, `Job`, `ReplicationController`, `ReplicaSet`, `DaemonSet`, `StatefulSet`, `CronJob`
