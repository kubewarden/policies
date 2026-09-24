# Containers Running In Privileged Mode

This Policy reports if containers are running in privileged mode. A privileged container is given access to all devices on the host. This allows the container nearly all the same access as processes running on the host.

By default a container is not allowed to access any devices on the host, but a "privileged" container is given access to all devices on the host. This allows the container nearly all the same access as processes running on the host. This is useful for containers that want to use linux capabilities like manipulating the network stack and accessing devices.


Look at the following path to see what the settings are. 
```
...
  spec:
    containers:
    - securityContext:
        privileged: <privilege>
```
https://kubernetes.io/docs/tasks/configure-pod-container/security-context/


# Settings
```yaml
  settings:
    parameters:
      - name: privilege
        type: boolean
        required: true
        value: false
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
