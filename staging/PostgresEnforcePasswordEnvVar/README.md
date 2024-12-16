# Postgres Enforce Environment Variable - POSTGRES_PASSWORD

This Policy ensures POSTGRES_PASSWORD environment variable are in place when using the official container images from Docker Hub.
POSTGRES_PASSWORD:   The POSTGRES_PASSWORD environment variable is required for you to use the PostgreSQL image. It must not be empty or undefined. This environment variable sets the superuser password for PostgreSQL. The default superuser is defined by the POSTGRES_USER environment variable.


If you encounter a violation, ensure the POSTGRES_PASSWORD environment variables is set.
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
