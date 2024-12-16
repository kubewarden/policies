# Missing Matchlabels Key

This Policy will enforce a custom match label key in `spec.selector`.


Add a custom label to `matchLabels`. 
```
...
  spec:
    selector:
      matchLabels:
        <label>: value
```
https://kubernetes.io/docs/concepts/overview/working-with-objects/labels/#resources-that-support-set-based-requirements


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

