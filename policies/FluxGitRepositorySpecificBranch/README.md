# GitRepository Specific Branch

GitRepository resources must reference a specific branch, e.g. 'main'.

Ensure the GitRepository ref field points to the specific branch (e.g. 'main').

# Settings
```yaml
  settings:
    parameters:
      - name: branch
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
`GitRepository`
