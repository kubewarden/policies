# Container Maximum Memory Requests

This Policy is to ensure a maximum Memory Request value is set. The policy will fail if the set Memory request is higher than the one specified in the policy. You must specify a unit size (e.g Mi, Gi, etc).


```
...
  spec:
    containers:
    - resources:
        requests:
          memory: <size>
```
https://kubernetes.io/docs/concepts/configuration/manage-resources-containers/


# Settings
```yaml
  settings:
    parameters:
      - name: size
        type: string
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

