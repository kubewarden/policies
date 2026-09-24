# Mongo-Express Enforce Environment Variable - ME_CONFIG_MONGODB_SERVER

This Policy ensures ME_CONFIG_MONGODB_SERVER environment variable are in place when using the official container images from Docker Hub.
ME_CONFIG_MONGODB_SERVER: The ME_CONFIG_MONGODB_SERVER environment variable sets the MongoDB container name. Use comma delimited list of host names for replica sets.


If you encounter a violation, ensure the ME_CONFIG_MONGODB_SERVER environment variables is set.
For futher information about the Mongo-Express Docker container, check here: https://hub.docker.com/_/mongo-express


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
