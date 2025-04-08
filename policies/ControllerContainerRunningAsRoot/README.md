# Container Running As Root

Running as root gives the container full access to all resources in the VM it is running on. Containers should not run with such access rights unless required by design. This Policy enforces that the `securityContext.runAsNonRoot` attribute is set to `true`.

You should set `securityContext.runAsNonRoot` to `true`. Not setting it will default to giving the container root user rights on the VM that it is running on.

```
...
  spec:
    securityContext:
      runAsNonRoot: true
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
