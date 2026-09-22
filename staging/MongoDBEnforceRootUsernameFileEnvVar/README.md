# MongoDB Enforce Environment Variable - MONGO_INITDB_ROOT_USERNAME_FILE

This Policy ensures MONGO_INITDB_ROOT_USERNAME_FILE environment variable are in place when using the official container images from Docker Hub.
MONGO_INITDB_ROOT_USERNAME_FILE: The MONGO_INITDB_ROOT_USERNAME_FILE environment variable is an alternative to passing sensitive information via environment variables, _FILE may be appended to the previously listed environment variables, causing the initialization script to load the values for those variables from files present in the container.


If you encounter a violation, ensure the MONGO_INITDB_ROOT_USERNAME_FILE environment variables is set.
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
