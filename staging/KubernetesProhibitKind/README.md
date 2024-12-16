# Prohibit Specific Kind From Being Scheduled

This Policy checks for a `kind` and can prevent it from being schedule to your cluster. A common example is running "naked" pods. 


Ensure you are not using a kind that is specified within the Policy.
```
kind: <kind>
```

https://kubernetes.io/docs/concepts/configuration/overview/#naked-pods-vs-replicasets-deployments-and-jobs


# Settings
```yaml
  settings:
    parameters:
      - name: kind
        type: string
        required: true
        value: Pods
```

# Resources
Policy applies to resources kinds:
`Pods`
