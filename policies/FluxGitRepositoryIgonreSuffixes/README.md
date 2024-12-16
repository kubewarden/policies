# GitRepository Ignore Suffixes

GitRepository resources must include specific suffixes in the spec.ignore field which determines the files to be ignored before making a commit.

Ensure the spec.ignore field in the GitRepository resource includes the required suffixes. The specified suffixes correspond to the file types that should not be included in the repository commit.

# Settings
```yaml
  settings:
    parameters:
      - name: suffixes
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
`GitRepository`
