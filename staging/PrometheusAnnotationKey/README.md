# Enforce Prometheus Annotation Key

This Policy ensures annotation keys like `prometheus.io/scrape`, `prometheus.io/port`, or `prometheus.io/path` are set in your Policy's metadata annotation. This is based on the default configuration of Prometheus.


Set annotation keys like `prometheus.io/scrape`, `prometheus.io/port`, or `prometheus.io/path`.
```
spec:
  template:
    metadata:
      annotations:
        <prometheus_annotation_key>: "value"
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

