# Container Maximum CPU Requests

This Policy is to ensure a maxiumum CPU Request value is set. The policy will fail if the set CPU request is higher than the one specified in the policy. You must specify a unit size (m, g, etc).


```
 ...
   spec:
     containers:
     - resources:
         requests:
           cpu: <size>
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

