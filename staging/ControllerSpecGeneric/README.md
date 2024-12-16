# Containers Are Missing A Controller Spec Key

This Policy will allow you to set your own Policies for the Pod spec. 


Ensure the key you have specified in your constraint exists in your workloads. 
```
...
  spec:
    <controller_spec_key>
```


# Settings
```yaml
  settings:
    parameters:
      - name: controller_spec_key
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

