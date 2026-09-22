# RabbitMQ Enforce Environment Variable - RABBITMQ_SSL_DEPTH

This Policy ensures RABBITMQ_SSL_DEPTH environment variable are in place when using the official container images from Docker Hub.
RABBITMQ_SSL_DEPTH: RABBITMQ_SSL_DEPTH is the maximum number of non-self-issued intermediate certificates that may follow the peer certificate in a valid certification path.


If you encounter a violation, ensure the RABBITMQ_SSL_DEPTH environment variables is set.
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
