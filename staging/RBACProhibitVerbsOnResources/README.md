# Rbac Prohibit Verbs On Resources

This Policy will violate if any RBAC ClusterRoles or Roles are designated with a specific Resource and Verb combination. For example, if you wanted to deny deployments, you would create a Policy with a resource of `deployments` and the verb `create`. 


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
    parameters:
      - name: resource
        type: string
        required: true
        value:
      - name: verb
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

