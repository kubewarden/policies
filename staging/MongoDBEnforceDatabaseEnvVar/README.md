# MongoDB Enforce Environment Variable - MONGO_INITDB_DATABASE

This Policy ensures MONGO_INITDB_DATABASE environment variable are in place when using the official container images from Docker Hub.
MONGO_INITDB_DATABASE: The MONGO_INITDB_DATABASE environment variable allows you to specify the name of a database to be used for creation scripts.


If you encounter a violation, ensure the MONGO_INITDB_DATABASE environment variables is set.
For futher information about the MongoDB Docker container, check here: https://hub.docker.com/_/mariadb


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
