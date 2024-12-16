# Persistent Volume Max Size

This Policy sets a max size on your Persistent Volumes. When specifying a size, be sure to include the size and sizing suffix (e.g. 1Gi).


Ensure the max size (e.g. 5Gi) on your persistent volume is no larger than what is specified in the Policy. 
```
spec:
  capacity:
    storage: <size>
```


# Settings
```yaml
  settings:
    parameters:
      - name: size
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

