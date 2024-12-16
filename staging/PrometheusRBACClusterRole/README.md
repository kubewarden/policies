# Prometheus Rbac Prohibit Verbs

This Policy blocks certain verbs from being set for Prometheus RBAC. This Policy applies to RBAC names containing the word `prometheus`. 


Check the `rules.verbs` for the verb(s) list and check the policy to see what value is set. 
```
metadata:
  name: prometheus
...
rules:
  - verbs:
      - <prometheus_verb>
```

https://kubernetes.io/docs/reference/access-authn-authz/rbac/


# Settings
```yaml
  settings:
    parameters:
      - name: prometheus_verb
        type: string
        required: true
        value: put
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
`ClusterRole`
