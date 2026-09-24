# Containers Using Hostpath

This Policy checks for containers that are trying to use HostPath. 

A `hostPath` volume mounts a file or directory from the host node's filesystem into your Pod. This is not something that most Pods will need, but it offers a powerful escape hatch for some applications.


Using HostPath could allow mounting the entire host’s filesystem into your pod, giving you read/write access on the host’s filesystem. This leaves your cluster vulnerable to escape Kubernetes constraints and access components at the Node (OS) level. 
```
...
  spec:
    template:
      spec:
        volumes:
          - hostPath:
```


# Settings
```yaml
  settings:
    parameters:
      - name: hostpath_key
        type: string
        required: true
        value: hostPath
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
