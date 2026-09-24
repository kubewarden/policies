# RabbitMQ Enforce Environment Variable - RABBITMQ_CONFIG_FILE

This Policy ensures RABBITMQ_CONFIG_FILE environment variable are in place when using the official container images from Docker Hub.
RABBITMQ_CONFIG_FILE: The path to the configuration file, without the .config extension. If the configuration file is present it is used by the server to configure RabbitMQ components. See Configuration guide for more information.


If you encounter a violation, ensure the RABBITMQ_CONFIG_FILE environment variables is set.
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
