# Rbac Prohibit Resources Wildcards

This Policy prohbits various resources from being set with wildcards for Role or ClusterRole resources, except for the `cluster-admin` ClusterRole. It will check each resource specified for the verb specified. The wildcards will be checked in:

### Resources
In the Kubernetes API, most resources are represented and accessed using a string representation of their object name, such as pods for a Pod. RBAC refers to resources using exactly the same name that appears in the URL for the relevant API endpoint. 

### Verbs
API verbs like get, list, create, update, patch, watch, delete, and deletecollection are used for resource requests. 

### API Groups
The API Group being accessed (for resource requests only).

### Non Resource URLs
Requests to endpoints other than /api/v1/... or /apis/<group>/<version>/... are considered "non-resource requests", and use the lower-cased HTTP method of the request as the verb.


Replace the `*` with the appropriate resource. The type of 
```
rules:
- <attributes>:
  - '*'
    
```
https://kubernetes.io/docs/reference/access-authn-authz/rbac/


# Settings
```yaml
  settings:
    parameters:
      - name: attributes
        type: string
        required: true
        value:
      - name: exclude_role_name
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

