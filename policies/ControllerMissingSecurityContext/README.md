# Containers Missing Security Context

This Policy checks if the container is missing securityContext while there is no securityContext defined on the Pod level as well. The security settings that are specified on the Pod level apply to all containers in the Pod.

Make sure you secure your containers by specifying a `securityContext` whether on each container or on the Pod level. The security settings that you specify on the Pod level apply to all containers in the Pod.

```
...
  spec:
    securityContext:
      <securityContext attributes>
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
