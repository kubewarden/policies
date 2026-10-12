# Namespace Pod Quota

When using a pod quota ensure setting the proper value for the quantity of pods you wish to have in your namespace.

Specify a value for the maximum number of pods you'd like in a namespace.

https://kubernetes.io/docs/tasks/administer-cluster/manage-resources/quota-pod-namespace/

# Settings

```yaml
settings:
  pod_quota: 2 # default: 2
  namespace: "magalix" # default: "magalix"
```

# Resources

Policy applies to resources kinds:
`ResourceQuota`
