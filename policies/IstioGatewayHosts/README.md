# Istio Gateway Approved Hosts

### Istio Gateway Approved Hosts
This ensures you are only serving traffic for approved hostnames. 


Ensure your domain name is the same as the one you are hosting.


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

