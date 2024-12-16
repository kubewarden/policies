# Containers Running With Privilege Escalation

Containers are running with PrivilegeEscalation configured. Setting this Policy to `true` allows child processes to gain more privileges than its parent process.  

This Policy gates whether or not a user is allowed to set the security context of a container to `allowPrivilegeEscalation` to `true`. The default value for this is `false` so no child process of a container can gain more privileges than its parent.

There are 2 parameters for this Policy:
- exclude_namespace (string) : This sets a namespace you want to exclude from Policy compliance checking. 
- allow_privilege_escalation (bool) : This checks for the value of `allowPrivilegeEscalation` in your spec.  


Check the following path to see what the PrivilegeEscalation value is set to.
```
...
  spec:
    containers:
      securityContext:
        allowPrivilegeEscalation: <value>
```
https://kubernetes.io/docs/tasks/configure-pod-container/security-context/


# Settings
```yaml
  settings:
    parameters:
      - name: exclude_namespaces
        type: array
        required: true
        value: [kube-system]
      - name: allow_privilege_escalation
        type: boolean
        required: true
        value: false
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
