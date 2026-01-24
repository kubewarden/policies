# Kubewarden probes validation policy

This policy validates that all containers have livenessProbe and readinessProbe
defined. As well as ensure that some basic configuration are defined on them.

## Settings

This policy configuration requires users to define if they want to enforce both
liveness and readiness probes or only one of them in the containers definition.

```yaml
settings:
  liveness:
    enforce: true
  readiness:
    enforce: true
```

User can also enforce some values configuration in all the period/thresold
probe's configuration. For that, inside each of the probe configuration, user
can define the minimum and limit values for each of the fields:

```yaml
settings:
  liveness:
    enforce: true
    failureThreshold: # optional
      minimum: 1
      limit: 5
    initialDelaySeconds: # optional
      minimum: 2
      limit: 3
    periodSeconds: # optional
      minimum: 1
      limit: 10
    successThreshold: # optional
      minimum: 3
      limit: 12
    terminationGracePeriodSeconds: # optional
      minimum: 1
      limit: 3
    timeoutSeconds: # optional
      minimum: 1
      limit: 10
  readiness:
    enforce: true
    failureThreshold: # optional
      minimum: 1
      limit: 5
    initialDelaySeconds: # optional
      minimum: 2
      limit: 3
    periodSeconds: # optional
      minimum: 1
      limit: 10
    successThreshold: # optional
      minimum: 3
      limit: 12
    terminationGracePeriodSeconds: # optional
      minimum: 1
      limit: 3
    timeoutSeconds: # optional
      minimum: 1
      limit: 10
```

Considering the above configuration. This will be considered a valid Pod:

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: valid-pod
spec:
  containers:
    - name: myapp
      image: busybox
      livenessProbe:
        exec:
          command: ["cat", "/tmp/healthy"]
        failureThreshold: 2
        initialDelaySeconds: 2
        periodSeconds: 2
        successThreshold: 3
        terminationGracePeriodSeconds: 2
        timeoutSeconds: 2
      readinessProbe:
        exec:
          command: ["cat", "/tmp/ready"]
        failureThreshold: 2
        initialDelaySeconds: 2
        periodSeconds: 2
        successThreshold: 3
        terminationGracePeriodSeconds: 2
        timeoutSeconds: 2
```

And the following one will be a invalid Pod:

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: invalid-pod
spec:
  containers:
    - name: myapp
      image: busybox
      # Missing livenessProbe
      readinessProbe:
        exec:
          command: ["cat", "/tmp/ready"]
        failureThreshold: 0 # Below minimum
        initialDelaySeconds: 1 # Below minimum
        periodSeconds: 11 # Above limit
        successThreshold: 2 # Below minimum
        terminationGracePeriodSeconds: 4 # Above limit
        timeoutSeconds: 0 # Below minimum
```
