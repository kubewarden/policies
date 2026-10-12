# Istio Injected Namespaces

# Istio-Injected Namespaces

Specify namespaces you would like to be labeld with `istio-injected: enabled`. Namespaces with this label with automatically deploy a Istio sidecar with each pod.

https://istio.io/latest/docs/setup/additional-setup/sidecar-injection/#automatic-sidecar-injection

# Settings

```yaml
settings:
  namespaces: []
```

# Resources

Policy applies to resources kinds:
`Namespace`
