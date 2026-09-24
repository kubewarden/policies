# Containers Should Not Run On Kubernetes Control Plane Nodes

Tolerations specified in the Policies for this template should not have any workloads scheduled on them. A common use case is the Kubernetes master. 


Check your tolerations against the Policy. 
```
...
  spec:
    tolerations:
    - key: <toleration_key>
```

https://kubernetes.io/docs/concepts/scheduling-eviction/taint-and-toleration/


# Settings
```yaml
  settings:
    parameters:
      - name: toleration_key
        type: string
        required: true
        value: node-role.kubernetes.io/master
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
