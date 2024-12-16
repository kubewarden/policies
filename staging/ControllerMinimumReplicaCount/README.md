# Containers Minimum Replica Count

Use this Policy to to check the replica count of your workloads. The value set in the Policy is greater than or equal to the amount desired, so if the replica count is lower than what is specified, the Policy will be in violation. 


The replica count should be a value equal or greater than what is set in the Policy.
```
spec:
  replicas: <replica_count>
```
https://kubernetes.io/docs/concepts/workloads/controllers/deployment/#scaling-a-deployment


# Settings
```yaml
  settings:
    parameters:
      - name: replica_count
        type: integer
        required: true
        value: 2
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
`Deployment`, `StatefulSet`, `ReplicaSet`, `ReplicationController`, `HorizontalPodAutoscaler`
