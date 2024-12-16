# Block Workloads Created Without Specifying Namespace

Using this Policy, you can prohibit workloads from being created in a default namespace due to the lack of a namespace label. 


Specify a `namespace` label. 
```
metadata:
  namespace:
```
https://kubernetes.io/docs/tasks/administer-cluster/namespaces/#understanding-the-motivation-for-using-namespaces


# Settings
```yaml
  settings:
    parameters:
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
