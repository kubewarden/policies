# Rbac Prohibit Wildcard On Secrets

This Policy will violate if any RBAC ClusterRoles or Roles are designated with 'wildcard' verb on 'secrets' resource.

When deploying RBAC roles, ensure the resource and verb combination you choose are allowed by the Policy.

```
rules:
- resources: <resources>
  verbs: <verb>
```

https://kubernetes.io/docs/reference/access-authn-authz/rbac/

# Settings

```yaml
settings:
  resource: "secrets" # default: "secrets"
  verb: "*" # default: "*"
  exclude_label_key: "" # optional, default: ""
  exclude_label_value: "" # optional, default: ""
```

# Resources

Policy applies to resources kinds:
`Role`, `ClusterRole`
