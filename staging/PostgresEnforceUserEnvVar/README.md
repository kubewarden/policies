# Postgres Enforce Environment Variable - POSTGRES_USER

This Policy ensures POSTGRES_USER environment variable are in place when using the official container images from Docker Hub.
POSTGRES_USER: The POSTGRES_USER environment variable is used in conjunction with POSTGRES_PASSWORD to set a user and its password. This variable will create the specified user with superuser power and a database with the same name. If it is not specified, then the default user of postgres will be used.
Be aware that if this parameter is specified, PostgreSQL will still show The files belonging to this database system will be owned by user "postgres" during initialization. This refers to the Linux system user (from /etc/passwd in the image) that the postgres daemon runs as, and as such is unrelated to the POSTGRES_USER option. See the section titled "Arbitrary --user Notes" for more details.


If you encounter a violation, ensure the POSTGRES_USER environment variables is set.
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
