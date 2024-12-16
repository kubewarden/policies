# Ingress Approved Hostnames

This Policy ensures only hostnames that are approved are set. Specify a list of approved hostnames since you may be hosting more than one domain name. 


Ensure your domain name is the same as the one you are hosting.
```
spec:
  rules:
  - host: <hostnames>
```


# Settings
```yaml
  settings:
    parameters:
      - name: hostnames
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

