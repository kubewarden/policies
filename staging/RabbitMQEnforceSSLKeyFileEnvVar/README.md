# RabbitMQ Enforce Environment Variable - RABBITMQ_SSL_KEYFILE

This Policy ensures RABBITMQ_SSL_KEYFILE environment variable are in place when using the official container images from Docker Hub.
RABBITMQ_SSL_KEYFILE: The RABBITMQ_SSL_KEYFILE sets a path for the server private key file.


If you encounter a violation, ensure the RABBITMQ_SSL_KEYFILE environment variables is set.
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
