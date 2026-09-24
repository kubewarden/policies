# Containers Running With Unapproved Linux Capabilities

Linux capabilities provide a finer-grained breakdown of the privileges traditionally associated with the superuser. Not specifying those capabilities gives the container access to all OS capabilities which may result in exploiting the VM at which the container is running. The issue is reported when a container has `SYS_ADMIN`, `NET_RAW`, `NET_ADMIN`, or `ALL` capabilities. For this Policy, you can also exclude a namespace, such as `kube-system`. 

With Linux capabilities, you can grant certain privileges to a process without granting all the privileges of the root user. To add or remove Linux capabilities for a Container, include the capabilities field in the securityContext section of the Container manifest.


You should set the specific Linux capabilities that your container needs. Or you could simply remove from `capabilities` the values of `SYS_ADMIN`, `NET_ADMIN`, and `ALL`.
```
...
  spec:
    containers:
    - securityContext:
        capabilities:
          add: ["SYS_ADMIN, "ALL", "NET_ADMIN"]
```
https://kubernetes.io/docs/tasks/configure-pod-container/security-context/


# Settings
```yaml
  settings:
    parameters:
      - name: exclude_namespaces
        type: array
        required: false
        value:
```

# Resources
Policy applies to resources kinds:
`Deployment`, `Job`, `ReplicationController`, `ReplicaSet`, `DaemonSet`, `StatefulSet`, `CronJob`
