# Container Custom Probe

This Policy allows defining the various container probes, and ensures consistent probes are configured across all containers. You can set the type of probe `livenessProbe`, `readinessProbe`, `startupProbe` along with a `command`, a `path` and a `port`. 


```
...
  spec:
    containers:
    - container1
      <probe_type>:
        exec:
          command: 
          - <command>
    - container2
      <probe_type>:
        httpGet:
          path: <path>
          port: <port>
```
https://kubernetes.io/docs/tasks/configure-pod-container/configure-liveness-readiness-startup-probes/#define-a-tcp-liveness-probe


# Settings
```yaml
  settings:
    parameters:
      - name: probe_type
        type: string
        required: true
        value:
      - name: command
        type: array
        required: false
        value:
      - name: path
        type: string
        required: false
        value:
      - name: port
        type: integer
        required: false
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

