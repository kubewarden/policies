# Containers Missing Security Context

This Policy checks if the container is missing securityContext while there is no securityContext defined on the Pod level as well. The security settings that are specified on the Pod level apply to all containers in the Pod.


Make sure you secure your containers by specifying a `securityContext` whether on each container or on the Pod level. The security settings that you specify on the Pod level apply to all containers in the Pod.
```
...
  spec:
    securityContext:
      <securityContext attributes>
```
https://kubernetes.io/docs/tasks/configure-pod-container/security-context/


# Settings
```yaml
  settings:
    parameters:
      - name: exclude_namespaces
        type: array
        required: false
        value: [kube-system]
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
