# Resource Suspend Waiver

Resource cannot be suspended unless it's on the waiver list.

Add the Resource to the waiver list or set 'suspend' to false.

# Settings
```yaml
  settings:
    parameters:
      - name: waiver_list
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
`HelmRelease`, `GitRepository`, `OCIRepository`, `Bucket`, `HelmChart`, `HelmRepository`, `Kustomization`
