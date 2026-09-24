# RabbitMQ Enforce Environment Variable - RABBITMQ_SSL_CACERTFILE

This Policy ensures RABBITMQ_SSL_CACERTFILE environment variable are in place when using the official container images from Docker Hub.
RABBITMQ_SSL_CACERTFILE: RABBITMQ_SSL_CACERTFILE sets a path to the CA certificate file. 


If you encounter a violation, ensure the RABBITMQ_SSL_CACERTFILE environment variables is set.
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
