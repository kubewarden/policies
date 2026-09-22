# Services Restrict Protocols

This Policy specifies what protocols should set for your Service. Any protocol not listed in this Policy will be in violation. 


Use a protocol that is specified in the Policy. 
```
spec:
  ports:
    - protocol: <protocols>
```


# Settings
```yaml
  settings:
    parameters:
      - name: protocols
        type: string
        required: true
        value: HTTPS
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
`Service`
