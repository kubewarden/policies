# Container Running As User

Containers has a feature in which you specify the ID of the user which all processes in the container will run with. This Policy enforces that the `securityContext.runAsUser` attribute is set to a uid greater than root uid. Running as root user gives the container full access to all resources in the VM it is running on. Containers should not run with such access rights unless required by design.

You should set `securityContext.runAsUser` uid to something greater than root uid. Not setting it will default to giving the container root user rights on the VM that it is running on.

```
...
  spec:
    securityContext:
      runAsUser: <uid>
```

https://kubernetes.io/docs/tasks/configure-pod-container/security-context/

# Settings

```yaml
settings:
  uid: 0 # default: 0
  exclude_namespaces: [] # optional, default: ["kube-system"]
  exclude_label_key: "" # optional
  exclude_label_value: "" # optional
```

# Resources

Policy applies to resources kinds:
`Deployment`, `Job`, `ReplicationController`, `ReplicaSet`, `DaemonSet`, `StatefulSet`, `CronJob`
