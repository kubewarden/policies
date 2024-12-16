# Prometheus Clusterrolebinding Has Incorrect Bindings

This Policy checks to see if the Prometheus Cluster Role Binding is bound to a `ClusterRole`, and is tied to a `ServiceAccount` name containing text of your choosing. The default policy is set for search for the Service Account name containing the word `prometheus`. 


Ensure the subject name you specify in the Policy matches what you are deploying. 
```
kind: ClusterRoleBinding
metadata:
  name: prometheus
...
subjects:
- kind: ServiceAccount
  name: prometheus
```
https://kubernetes.io/docs/reference/access-authn-authz/rbac/


# Settings
```yaml
  settings:
    parameters:
      - name: prometheus_subject_name
        type: string
        required: true
        value: prometheus
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
