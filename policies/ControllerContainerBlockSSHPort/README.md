# Containers Block Ssh Port

This Policy checks if the container is exposing ssh port.

Make sure you are not exposing ssh port on containers.

```
...
  spec:
    containers:
      ports:
      - containerPort: <port>
```

# Settings

```yaml
settings:
  container_port: 22 # default: 22
  exclude_namespaces: [] # optional
  exclude_label_key: "" # optional
  exclude_label_value: "" # optional
```

# Resources

Policy applies to resources kinds:
`Deployment`, `Job`, `ReplicationController`, `ReplicaSet`, `DaemonSet`, `StatefulSet`, `CronJob`
