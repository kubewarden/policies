# Containers Enforce Restart Policy

This Policy checks if a specific restartPolicy is configured in your workloads.


Ensure the restartPolicy is set to some specific policy in your workloads. 
```
...
  spec:
    restartPolicy: <policy>
```


# Settings
```yaml
  settings:
    parameters:
      - name: restart_policy
        type: string
        required: true
        value: Always
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
