# Resource Quota Setting Is Missing

When creating resource quotas per namespace, ensure CPU and Memory requests and limits are set.


Declare values for CPU & Memory requests and limits.  
```
spec:
  hard:
    requests.cpu: <size>
    requests.memory: <size>
    limits.cpu: <size>
    limits.memory: <size>
```
https://kubernetes.io/docs/tasks/administer-cluster/manage-resources/quota-memory-cpu-namespace/


# Settings
```yaml
  settings:
    parameters:
      - name: resource_type
        type: string
        required: true
        value:
      - name: namespace
        type: string
        required: true
        value: magalix
```

# Resources
Policy applies to resources kinds:

