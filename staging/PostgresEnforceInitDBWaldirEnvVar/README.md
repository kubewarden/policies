# Postgres Enforce Environment Variable - POSTGRES_INITDB_WALDIR

This Policy ensures POSTGRES_INITDB_WALDIR environment variable are in place when using the official container images from Docker Hub.
POSTGRES_INITDB_WALDIR: The POSTGRES_INITDB_WALDIR environment variable is used to define another location for the Postgres transaction log. By default the transaction log is stored in a subdirectory of the main Postgres data folder (PGDATA). 


If you encounter a violation, ensure the POSTGRES_INITDB_WALDIR environment variables is set.
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
