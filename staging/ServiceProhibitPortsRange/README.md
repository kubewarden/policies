# Services Prohibit Ports Range

This Policy checks if ports allocated for your services are using a number that is less than the specified value. 


Use a port that is greater than or equal to what is specified in the Policy. 
```
spec:
  ports:
    - targetPort: <target_port>
```


# Settings
```yaml
  settings:
    parameters:
      - name: target_port
        type: integer
        required: true
        value: 1024
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
