# Rbac Protect Cluster Admin Clusterrolebindings

This Policy allows you to select which groups you can set for Cluster-admin. The default policy checks for the 

```
subjects:
- kind: Group
  name: system:masters
```

`cluster-admin` allows super-user access to perform any action on any resource. When used in a ClusterRoleBinding, it gives full control over every resource in the cluster and in all namespaces. When used in a RoleBinding, it gives full control over every resource in the role binding's namespace, including the namespace itself. Be selective when adding additional subjects. 


Remove any kinds and names that are not consistent with the constraint. 
```
kind: ClusterRoleBinding
metadata:
  name: cluster-admin
...
subjects:
- kind: Group
  name: system:masters
```  
https://kubernetes.io/docs/reference/access-authn-authz/rbac/


# Settings
```yaml
  settings:
    parameters:
      - name: subjects_name
        type: string
        required: true
        value: system:masters
      - name: subjects_kind
        type: string
        required: true
        value: Group
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
`ClusterRoleBinding`
