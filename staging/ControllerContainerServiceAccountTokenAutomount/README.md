# Controller ServiceAccount Tokens Automount

This Policy allows for the option of enabling or disabling Service Accounts that are created for a Pod. The recommended practice is to set the `automount_token` to `false.  

When a pod is created without specifying a service account, it is automatically assigned the default service account in the same namespace. This is a security concern because a compromised container can access the API using automatically mounted service account credentials. The API permissions of the service account depend on the authorization plugin and policy in use, but could possibly create and delete pods. 

We recommend setting the `automount_token` to `false`. 

In version 1.6+, you can opt out of automounting API credentials for a particular pod.


Ensure the setting in the Policy matches the Service Account declaration in the controller. 
```
...
  spec:
    automountServiceAccountToken: false
```

https://kubernetes.io/docs/tasks/configure-pod-container/configure-service-account/#use-the-default-service-account-to-access-the-api-server


# Settings
```yaml
  settings:
    parameters:
      - name: automount_token
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
