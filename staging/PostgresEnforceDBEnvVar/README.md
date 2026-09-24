# Postgres Enforce Environment Variable - POSTGRES_DB

This Policy ensures POSTGRES_DB environment variable are in place when using the official container images from Docker Hub.
POSTGRES_DB: The POSTGRES_DB environment variable defines a different name for the default database that is created when the image is first started. If it is not specified, then the value of POSTGRES_USER will be used.


If you encounter a violation, ensure the POSTGRES_DB environment variables is set.
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
