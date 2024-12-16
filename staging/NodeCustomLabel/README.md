# Node Custom Label

This Policy checks for custom labels on your node. This Policy looks for both a key and value. 


Add the custom label specified in the policy to all nodes. 
```
metadata: 
  labels: 
      <key>: <value>
```


# Settings
```yaml
  settings:
    parameters:
      - name: key
        type: string
        required: true
        value:
      - name: value
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

