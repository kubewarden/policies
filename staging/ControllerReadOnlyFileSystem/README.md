# Containers Read Only Root Filesystem

This Policy will cause a violation if the root file system is not mounted as specified. As a security practice, the root file system should be read-only or expose risk to your nodes if compromised. 

This Policy requires containers must run with a read-only root filesystem (i.e. no writable layer).


Set `readOnlyRootFilesystem` in your `securityContext` to the value specified in the Policy. 
```
...
  spec:
    containers:
      - securityContext:
          readOnlyRootFilesystem: <read_only>
```

https://kubernetes.io/docs/concepts/policy/pod-security-policy/#volumes-and-file-systems


# Settings
```yaml
  settings:
    parameters:
      - name: read_only
        type: boolean
        required: true
        value: true
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
