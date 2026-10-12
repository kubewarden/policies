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
  restricted_image_names: []
  exclude_namespaces: [] # optional
  exclude_label_key: "" # optional
  exclude_label_value: "" # optional
```

# Resources
Policy applies to resources kinds:
`CronJob`, `DaemonSet`, `Deployment`, `Job`, `Pod`, `StatefulSet`
