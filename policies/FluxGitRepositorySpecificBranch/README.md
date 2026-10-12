# GitRepository Specific Branch

GitRepository resources must reference a specific branch, e.g. 'main'.

Ensure the GitRepository ref field points to the specific branch (e.g. 'main').

# Settings

```yaml
settings:
  branch: "branch"
  exclude_namespaces: [] # optional
  exclude_label_key: "" # optional
  exclude_label_value: "" # optional
```

# Resources

Policy applies to resources kinds:
`GitRepository`
