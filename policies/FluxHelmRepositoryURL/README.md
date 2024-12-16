# Helm Repo URL Should Be in Organisation Domain

The URL of a Helm repository should only be from the specified organisation domain.

Change the URL of the Helm repository to one that is from the organisation domain.

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
`HelmRepository`
