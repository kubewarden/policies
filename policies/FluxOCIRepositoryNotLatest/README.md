# OCIRepository Not Latest Tag

OCIRepository resources must not use 'latest' as a tag reference.

Specify a versioned tag for the OCIRepository instead of using 'latest'.

# Settings
```yaml
  settings:
    parameters:
      - name: exclude_namespaces
        type: array
        required: false
        value: []
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
`OCIRepository`
