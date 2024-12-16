# GitRepository Organization Domains

GitRepository resources must only be from allowed organization domains.

Ensure the GitRepository URL belongs to an allowed organization domain.

# Settings
```yaml
  settings:
    parameters:
      - name: domains
        type: array
        required: true
        value:
      - name: exclude_namespaces
        type: array
        required: false
        value: []
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
`GitRepository`
