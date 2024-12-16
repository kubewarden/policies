# GitRepository Untrusted Domains

GitRepository resources must not use targets from untrusted domains.

Ensure the GitRepository URL does not belong to an untrusted domain.

# Settings
```yaml
  settings:
    parameters:
      - name: untrusted_domains
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
