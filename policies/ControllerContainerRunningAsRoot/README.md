# Container Running As Root

Running as root gives the container full access to all resources in the VM it is running on. Containers should not run with such access rights unless required by design. This Policy enforces that the `securityContext.runAsNonRoot` attribute is set to `true`. 


You should set `securityContext.runAsNonRoot` to `true`. Not setting it will default to giving the container root user rights on the VM that it is running on. 
```
...
  spec:
    securityContext:
      runAsNonRoot: true
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
