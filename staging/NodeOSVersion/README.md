# Node OS Version

This Policy allows enforcement of the type of OS image used for each node. Build consistency throughout your cluster by provisioning the same OS version across all the nodes. 


When provisioning new Kubernetes nodes, ensure your provisiong process specifies an operating system that is equal to the OS Image listed in the Policy. 
```
status:
  nodeInfo:
    osImage: <os>
```


# Settings
```yaml
  settings:
    parameters:
      - name: os
        type: string
        required: true
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

