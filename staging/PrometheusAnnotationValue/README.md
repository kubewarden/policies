# Enforce Prometheus Annotation Value

This Policy ensures custom annotation values are set like Prometheus scrape, path, and port values in your Prometheus annotations. If you are using something other than the default values, you can customize those requirements here. 


Ensure the values in the Policy match your custom Prometheus configurations. 
```
metadata:
  annotations:
    <prometheus_annotation_key>: <prometheus_annotation_value>
```
https://prometheus.io/docs/prometheus/latest/configuration/configuration/#pod


# Settings
```yaml
  settings:
    parameters:
      - name: prometheus_annotation_key
        type: string
        required: true
        value:
      - name: prometheus_annotation_value
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

