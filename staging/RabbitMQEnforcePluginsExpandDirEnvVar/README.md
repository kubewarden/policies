# RabbitMQ Enforce Environment Variable - RABBITMQ_PLUGINS_EXPAND_DIR

This Policy ensures RABBITMQ_PLUGINS_EXPAND_DIR environment variable are in place when using the official container images from Docker Hub.
RABBITMQ_PLUGINS_EXPAND_DIR: Working directory used to expand enabled plugins when starting the server. It is important that effective RabbitMQ user has sufficient permissions to read and create files and subdirectories in this directory.


If you encounter a violation, ensure the RABBITMQ_PLUGINS_EXPAND_DIR environment variables is set.
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
