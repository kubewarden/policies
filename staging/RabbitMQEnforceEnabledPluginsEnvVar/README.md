# RabbitMQ Enforce Environment Variable - RABBITMQ_ENABLED_PLUGINS_FILE

This Policy ensures RABBITMQ_ENABLED_PLUGINS_FILE environment variable are in place when using the official container images from Docker Hub.
RABBITMQ_ENABLED_PLUGINS_FILE: This file records explicitly enabled plugins. When a plugin is enabled or disabled, this file will be recreated. It is important that effective RabbitMQ user has sufficient permissions to read, write and create this file at any time.


If you encounter a violation, ensure the RABBITMQ_ENABLED_PLUGINS_FILE environment variables is set.
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
