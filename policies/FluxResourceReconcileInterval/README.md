# Resource Reconcile Interval Must Be Set Between Lower and Upper Bound

The reconcile interval of a Resource must be set between a lower and upper bound, lower_bound & upper_bound must be in seconds .

Set the reconcile interval of the Resource between the specified lower and upper bounds.

# Settings
```yaml
  settings:
    parameters:
      - name: lower_bound
        type: integer
        required: true
        value: 1
      - name: upper_bound
        type: integer
        required: true
        value: 1000
      - name: exclude_namespaces
        type: array
        required: false
        value: []
      - name: exclude_label_key
        type: string
        required: false
        value: ""
      - name: exclude_label_value
        type: string
        required: false
        value: ""
```

# Resources
Policy applies to resources kinds:
`HelmRelease`, `GitRepository`, `OCIRepository`, `Bucket`, `HelmChart`, `HelmRepository`, `Kustomization`
