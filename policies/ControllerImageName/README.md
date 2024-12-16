# Containers Block Specific Image Names

This Policy prohibits images with certain image names from being deployed by specifying a list of unapproved names. 


Use an image that is not set in the Policy. 
```
...
  spec:
    containers:
    - image: registry/<image_names>:tag
```


# Settings
```yaml
  settings:
    parameters:
      - name: restricted_image_names
        type: array
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

