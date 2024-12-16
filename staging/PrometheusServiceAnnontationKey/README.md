# Enforce Prometheus Service Annotation Key

This Policy checks to see if Services have the specified Prometheus annotations like  `prometheus.io/scrape` and `filter.by.port.name`. These annotations are used to send metric data to your Prometheus instance. 


```
kind: Service
metadata:
  annotations:
    <prometheus_service_annotation>: value
```

https://prometheus.io/docs/prometheus/latest/configuration/configuration/#scrape_config


# Settings
```yaml
  settings:
    parameters:
      - name: prometheus_service_annotation
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

