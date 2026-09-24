# Services Prohibit Type

This Policy checks your Kubernetes Service kind to see what Service type is set. If a specified service type is found, this Policy will be in violation. Security practices suggest using types `ServiceType` of `ClusterIP` or `LoadBalancer` and not `NodePort`. 


Ensure the type matches what is specified in the Policy. 
```
spec:
  type: <type>
```

https://kubernetes.io/docs/concepts/services-networking/service/#nodeport


# Settings
```yaml
  settings:
    parameters:
      - name: type
        type: string
        required: true
        value: NodePort
      - name: exclude_namespaces
        type: array
        required: false
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
`Service`
