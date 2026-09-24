# Containers Sharing Process Namespace

This Policy allows check if sharing process namespace with other containers in the pod should be allowed or not. Resources that can be shared with the container include:

### hostNetwork
Controls whether the pod may use the node network namespace. Doing so gives the pod access to the loopback device, services listening on localhost, and could be used to snoop on network activity of other pods on the same node.

### hostPID
Controls whether the pod containers can share the host process ID namespace. Note that when paired with ptrace this can be used to escalate privileges outside of the container (ptrace is forbidden by default).

### shareProcessNamespace
When process namespace sharing is enabled, processes in a container are visible to all other containers in that pod.

### hostIPC
Controls whether the pod containers can share the host IPC namespace.


Match the shared resource with either true or false, as set in your constraint. 
```
...
  spec:
    <shared_resource>: <resource_enabled>
```
https://kubernetes.io/docs/concepts/policy/pod-security-policy/#host-namespaces


# Settings
```yaml
  settings:
    parameters:
      - name: resource_enabled
        type: boolean
        required: true
        value: false
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
