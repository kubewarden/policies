# RabbitMQ Enforce Environment Variable - RABBITMQ_DEFAULT_USER

This Policy ensures RABBITMQ_DEFAULT_USER environment variable are in place when using the official container images from Docker Hub.
RABBITMQ_DEFAULT_USER: The RABBITMQ_DEFAULT_USER environment variable sets the User name to create when RabbitMQ creates a new database from scratch.


If you encounter a violation, ensure the RABBITMQ_DEFAULT_USER environment variables is set.
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
