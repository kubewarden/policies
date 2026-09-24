# Postgres Enforce Environment Variable - POSTGRES_HOST_AUTH_METHOD

This Policy ensures POSTGRES_HOST_AUTH_METHOD environment variable are in place when using the official container images from Docker Hub.
POSTGRES_HOST_AUTH_METHOD: The POSTGRES_HOST_AUTH_METHOD environment variable is used to control the auth-method for host connections for all databases, all users, and all addresses. If unspecified then md5 password authentication is used. 


If you encounter a violation, ensure the POSTGRES_HOST_AUTH_METHOD environment variables is set.
For futher information about the Postgres Docker container, check here: https://hub.docker.com/_/postgres


# Settings
```yaml
  settings:
    parameters:
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
`Deployment`, `Job`, `ReplicationController`, `ReplicaSet`, `DaemonSet`, `StatefulSet`, `CronJob`
