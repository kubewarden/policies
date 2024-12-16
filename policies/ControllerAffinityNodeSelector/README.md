# Affinity Node Selector

This Policy allows setting a key and value for `nodeSelector` when assigning pods to nodes. 

`nodeSelector` is a field of PodSpec. It specifies a map of key-value pairs. For the pod to be eligible to scheduled on a node, the node must have each of the indicated key-value pairs as labels (it can have additional labels as well). 


When working with with `nodeSelector`, the indicated key-value pair will be matched to a node label.  

```
...
  spec:
    nodeSelector:
      <key>: <value>
```
https://kubernetes.io/docs/concepts/scheduling-eviction/assign-pod-node/#nodeselector


# Settings
```yaml
  settings:
    parameters:
      - name: key
        type: string
        required: true
        value:
      - name: value
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

