# Resource Suspend Waiver

Resource cannot be suspended unless it's on the waiver list.

Add the Resource to the waiver list or set 'suspend' to false.

# Settings

```yaml
settings:
  waiver_list: []
  exclude_namespaces: [] # optional, default: []
  exclude_label_key: "" # optional
  exclude_label_value: "" # optional
```

# Resources

Policy applies to resources kinds:
`HelmRelease`, `GitRepository`, `OCIRepository`, `Bucket`, `HelmChart`, `HelmRepository`, `Kustomization`
