# Kubernetes Minimum Kubelet Version

This Policy checks the minimum kubelet version for your Nodes. Please use this type of versioning (`v1.0.0`) and omit any suffix starting with a hyphen (`-`)


Upgrade your kubelet to a version that is equal to or later than the version specified in the policy. 
```
nodeInfo:
  kubeletVersion: <version>
```


# Settings
```yaml
  settings:
    parameters:
      - name: version
        type: string
        required: true
        value:
```

# Resources
Policy applies to resources kinds:

