# Prohibit Naked Pods From Being Scheduled

This Policy checks for a `kind` and can prohibit it from being schedule to your cluster. A common example is running "naked" pods. 


Ensure you are not using a kind that is specified within the Policy.
```
kind: <kind>
```

https://kubernetes.io/docs/concepts/configuration/overview/#naked-pods-vs-replicasets-deployments-and-jobs


# Settings
```yaml
  settings:
    parameters: null
```

# Resources
Policy applies to resources kinds:
`Pod`
