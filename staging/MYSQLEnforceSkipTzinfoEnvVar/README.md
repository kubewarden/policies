# MYSQL Enforce Environment Variable - MYSQL_INITDB_SKIP_TZINFO

This Policy ensures MYSQL_INITDB_SKIP_TZINFO environment variable are in place when using the official container images from Docker Hub.
MYSQL_INITDB_SKIP_TZINFO: The MYSQL_INITDB_SKIP_TZINFO environment variable allows the skipping of timezone checking when initializing the DB.


If you encounter a violation, ensure the MYSQL_INITDB_SKIP_TZINFO environment variables is set.
For futher information about the MYSQL Docker container, check here: https://hub.docker.com/_/mysql


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
