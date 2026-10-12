# Resource Reconcile Interval Must Be Set Between Lower and Upper Bound

The reconcile interval of a Resource must be set between a lower and upper bound, lower_bound & upper_bound must be in seconds .

Set the reconcile interval of the Resource between the specified lower and upper bounds.

# Settings

```yaml
settings:
  lower_bound: 1 # default: 1
  upper_bound: 1000 # default: 1000
  exclude_namespaces: [] # optional, default: []
  exclude_label_key: "" # optional, default: ""
  exclude_label_value: "" # optional, default: ""
```

# Resources

Policy applies to resources kinds:
`HelmRelease`, `GitRepository`, `OCIRepository`, `Bucket`, `HelmChart`, `HelmRepository`, `Kustomization`
