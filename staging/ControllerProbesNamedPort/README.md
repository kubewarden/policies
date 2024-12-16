# Container Named Port

This Policy checks the named port of your containers to see if they are set in your container probes. 


https://kubernetes.io/docs/tasks/configure-pod-container/configure-liveness-readiness-startup-probes/#define-startup-probes


# Settings
```yaml
  settings:
    parameters:
      - name: probe
        type: string
        required: true
        value: livenessProbe
```

# Resources
Policy applies to resources kinds:

