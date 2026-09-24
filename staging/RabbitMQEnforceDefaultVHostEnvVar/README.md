# RabbitMQ Enforce Environment Variable - RABBITMQ_DEFAULT_VHOST

This Policy ensures RABBITMQ_DEFAULT_VHOST environment variable are in place when using the official container images from Docker Hub.
RABBITMQ_DEFAULT_VHOST: RABBITMQ_DEFAULT_VHOST sets a Virtual host to create from scratch.


If you encounter a violation, ensure the RABBITMQ_DEFAULT_VHOST environment variables is set.
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
