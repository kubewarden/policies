# Containers Should Not Run In Namespace

This Policy ensure workloads are not running in a specified namespace. 


Use a `namespace` that differs from the one specified in the Policy. 
```
metadata:
  namespace: <custom_namespace>
```

https://kubernetes.io/docs/concepts/overview/working-with-objects/namespaces/


# Settings
```yaml
  settings:
    parameters:
      - name: custom_namespace
        type: string
        required: true
        value: default
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
`Pod`, `Deployment`, `Job`, `ReplicationController`, `ReplicaSet`, `DaemonSet`, `StatefulSet`, `CronJob`
