# Postgres Enforce Environment Variable - POSTGRES_INITDB_ARGS

This Policy ensures POSTGRES_INITDB_ARGS environment variable are in place when using the official container images from Docker Hub.
POSTGRES_INITDB_ARGS:   The POSTGRES_INITDB_ARGS environment variable is used to send arguments to postgres initdb. The value is a space separated string of arguments as postgres initdb would expect them. This is useful for adding functionality like data page checksums: `-e POSTGRES_INITDB_ARGS="--data-checksums"`.


If you encounter a violation, ensure the POSTGRES_INITDB_ARGS environment variables is set.
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
