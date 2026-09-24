# Mongo-Express Enforce Environment Variable - ME_CONFIG_REQUEST_SIZE

This Policy ensures ME_CONFIG_REQUEST_SIZE environment variable are in place when using the official container images from Docker Hub.
ME_CONFIG_REQUEST_SIZE: The ME_CONFIG_REQUEST_SIZE environment variable sets the maximum payload size. CRUD operations above this size will fail in [body-parser](https://www.npmjs.com/package/body-parser).


If you encounter a violation, ensure the ME_CONFIG_REQUEST_SIZE environment variables is set.
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
