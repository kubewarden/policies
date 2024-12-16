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
    parameters:
      - name: container_port
        type: integer
        required: true
        value: 22
      - name: exclude_namespaces
        type: array
        required: false
        value:
      - name: exclude_label_key
        type: string
        required: false
        value:
      - name: exclude_label_value
        type: string
        required: false
        value:
```

# Resources
Policy applies to resources kinds:
`Deployment`, `Job`, `ReplicationController`, `ReplicaSet`, `DaemonSet`, `StatefulSet`, `CronJob`
