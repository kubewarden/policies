# Container Enforce Environment Variable

This Policy checks whether an environment variable is missing or not. 


If you encounter a violation, ensure the environment variable is set. 


# Settings
```yaml
  settings:
    parameters:
      - name: envvar_name
        type: string
        required: true
        value:
      - name: app_name
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

