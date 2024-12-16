# Containers Are Missing A Spec Template Label

This Policy allows setting a custom label in `spec.template.metadata`.


Add a custom label to `spec.template.metadata.label` as specified in your constraint.  
```
...
  spec:
    template:
      metadata:
        labels:
          <label>
```
https://kubernetes.io/docs/concepts/workloads/controllers/deployment/#selector


# Settings
```yaml
  settings:
    parameters:
      - name: label
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

