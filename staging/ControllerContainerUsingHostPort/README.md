# Containers Using Hostport

This Policy checks if `hostPort` is set. When `hostPort` is set, a Pod is bound to a hostPort and limits the number of places the Pod can be scheduled. That's because each <hostIP, hostPort, protocol> combination must be unique. If you don't specify the hostIP and protocol explicitly, Kubernetes will use 0.0.0.0 as the default hostIP and TCP as the default protocol.

Don't specify a hostPort for a Pod unless it is absolutely necessary.  


Try avoid setting `hostPort` in your spec. 
```
...
  spec:
    containers:
    - ports:
      - hostPort: 8080
```
https://kubernetes.io/docs/concepts/configuration/overview/#services


# Settings
```yaml
  settings:
    parameters:
      - name: host_port
        type: string
        required: true
        value: hostPort
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
