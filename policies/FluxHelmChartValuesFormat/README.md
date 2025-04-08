# HelmChart Values File Format

HelmChart must reference values files in the following format: 'xxx=values.yaml'.

Update the HelmChart valuesFrom field to use the correct format.

# Settings

```yaml
settings:
  exclude_namespaces: [] # optional
  exclude_label_key: "" # optional
  exclude_label_value: "" # optional
```

# Resources

Policy applies to resources kinds:
`HelmChart`
