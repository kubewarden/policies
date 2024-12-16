# Namespace Pod Quota

When using a pod quota ensure setting the proper value for the quantity of pods you wish to have in your namespace. 


Specify a value for the maximum number of pods you'd like in a namespace. 

https://kubernetes.io/docs/tasks/administer-cluster/manage-resources/quota-pod-namespace/


# Settings
```yaml
  settings:
    parameters:
      - name: pod_quota
        type: string
        required: true
        value: 2
      - name: namespace
        type: string
        required: true
        value: magalix
```

# Resources
Policy applies to resources kinds:

