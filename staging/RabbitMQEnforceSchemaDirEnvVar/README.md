# RabbitMQ Enforce Environment Variable - RABBITMQ_SCHEMA_DIR

This Policy ensures RABBITMQ_SCHEMA_DIR environment variable are in place when using the official container images from Docker Hub.
RABBITMQ_SCHEMA_DIR: The directory where RabbitMQ keeps its configuration schema used by the new style configuration file.


If you encounter a violation, ensure the RABBITMQ_SCHEMA_DIR environment variables is set.
For futher information about the RabbitMQ Docker container, check here: https://hub.docker.com/_/rabbitmq


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
