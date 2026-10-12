# Network Allow Ingress Traffic From Namespace To Another

If you are using a CNI that allows for Network Policies, you can use this Policy to allow Ingress traffic from one namespace to another.

By default, if no policies exist in a namespace, then all ingress and egress traffic is allowed to and from pods in that namespace.

Validate your use case and check network policies for traffic blocking.

https://kubernetes.io/docs/concepts/services-networking/network-policies/

# Settings

```yaml
settings:
  src_namespace: "src_namespace"
  dst_namespace: "dst_namespace"
  exclude_label_key: "" # optional
  exclude_label_value: "" # optional
```

# Resources

Policy applies to resources kinds:
`NetworkPolicy`
