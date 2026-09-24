# RabbitMQ Enforce Environment Variable - RABBITMQ_MNESIA_DIR

This Policy ensures RABBITMQ_MNESIA_DIR environment variable are in place when using the official container images from Docker Hub.
RABBITMQ_MNESIA_DIR: The directory where this RabbitMQ node's data is stored. This s a schema database, message stores, cluster member information and other persistent node state.


If you encounter a violation, ensure the RABBITMQ_MNESIA_DIR environment variables is set.
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
