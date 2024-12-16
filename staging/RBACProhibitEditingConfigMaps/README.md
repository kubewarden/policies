# Rbac Prohibit Updating Configmaps

This Policy will violate if any RBAC ClusterRoles or Roles are designated with a specific verbs set for ConfigMaps. For example, if you wanted to deny the patching of a configmap, you would create a Policy with a the `resource_name` of the configmap you want to protect, it's `namespace` and the `verb` you want to block. 


When deploying RBAC roles, ensure the `resourceName`, `namespace` and `verb combination` you choose are allowed by the Policy. 
```
rules:
- resourcesName: <resource_name>
  verbs: <verb>

```

https://kubernetes.io/docs/reference/access-authn-authz/rbac/


# Settings
```yaml
  settings:
    parameters:
      - name: resource_name
        type: string
        required: true
        value:
      - name: verb
        type: string
        required: true
        value:
      - name: namespace
        type: string
        required: true
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

