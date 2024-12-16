# Helm Repo Type Should Be OCI

The type of a Helm repository should be OCI.

Change the type of the Helm repository to OCI.

# Settings
```yaml
  settings:
    parameters:
      - name: exclude_namespaces
        type: array
        required: false
        value: ["kube-system"]
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
