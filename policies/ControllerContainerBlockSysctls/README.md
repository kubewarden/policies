# Container Block Sysctls

Setting sysctls can allow containers unauthorized escalated privileges to a Kubernetes node. 


You should not set  `securityContext.sysctls` 
```
...
  spec:
    securityContext:
      sysctls
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
