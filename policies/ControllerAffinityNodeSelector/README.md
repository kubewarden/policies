# Affinity Node Selector

This Policy allows setting a key and value for `nodeSelector` when assigning pods to nodes.

`nodeSelector` is a field of PodSpec. It specifies a map of key-value pairs. For the pod to be eligible to scheduled on a node, the node must have each of the indicated key-value pairs as labels (it can have additional labels as well).

When working with `nodeSelector`, the indicated key-value pair will be matched to a node label.

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
  key: "key"
  value: "value"
  exclude_namespaces: [] # optional
  exclude_label_key: "" # optional
  exclude_label_value: "" # optional
```

# Resources

Policy applies to resources kinds:
`CronJob`, `DaemonSet`, `Deployment`, `Job`, `Pod`, `StatefulSet`
