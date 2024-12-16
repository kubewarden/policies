# Metadata Missing Label And Value

Custom labels can help enforce organizational standards for each artifact deployed. This Policy ensures a key and value are set in the entity's `metadata.labels` path. The Policy detects the presence of the following: 

### label
A label key of your choosing. 

### value
A label value of your choosing.


Add a custom label and value to `metadata.labels`.

```
metadata:
  labels:
    <label>: <value>
```  
For additional information, please check
* https://kubernetes.io/docs/concepts/overview/working-with-objects/common-labels 


# Settings
```yaml
  settings:
    parameters:
      - name: label
        type: string
        required: true
        value:
      - name: value
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

