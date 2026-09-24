# Disable ServiceAccount Token Automount In Specific Namespace

This Policy allows you to enforce the enabling or disabling the automounting of service account tokens. 

When a pod is created without a service account defined, the default service account within the same namespace will be assigned automatically. 

This is a security concern because a kubernetes client can load a container's service account token. With that token a compromoised contaienr can then access the Kubernetes API to perform actions such as creating and deleting pods.

In version 1.6+, you can opt out of automounting API credentials for a service account by setting automountServiceAccountToken: false on the service account.


Add the key:value pair `automountServiceAccountToken: false` to your Service Account declaration. 
```
kind: ServiceAccount
automountServiceAccountToken: false
```

https://kubernetes.io/docs/tasks/configure-pod-container/configure-service-account/#use-the-default-service-account-to-access-the-api-server


# Settings
```yaml
  settings:
    parameters:
      - name: automount
        type: boolean
        required: true
        value: false
      - name: namespace
        type: string
        required: true
        value: default
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
`ServiceAccount`
