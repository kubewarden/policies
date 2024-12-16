# Enforce Specific Ingress Class

This Policy allows for a custom Ingress annotation key and value. When using Kubernetes Ingress, various annotation can be used to define different configurations, even different ingress controllers. Values may vary depending on your use case. 


```
metadata:
  annotations:
    kubernetes.io/ingress.class: <class>
```
https://kubernetes.io/docs/concepts/services-networking/ingress/


# Settings
```yaml
  settings:
    parameters:
      - name: class
        type: string
        required: true
        value:
      - name: annotation
        type: string
        required: true
        value:
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

