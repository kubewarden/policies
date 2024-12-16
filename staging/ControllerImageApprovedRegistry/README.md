# Approved Registry

This Policy is violated when container images are not using approved container registries. The Policy allows for you to add multiple registries. 


Ensure the image registry you are using matches the registries specified in the policy. If you are using multiple containers, be sure to check those as well. 

Replace the violating registry with one that is approved. 
```
...
  spec:
    template:
      spec:
        containers:
        - image:<registries>/container_name:tag
```


# Settings
```yaml
  settings:
    parameters:
      - name: registries
        type: array
        required: true
        value:
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

