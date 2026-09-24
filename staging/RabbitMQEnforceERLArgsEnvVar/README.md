# RabbitMQ Enforce Environment Variable - RABBITMQ_SERVER_ADDITIONAL_ERL_ARGS

This Policy ensures RABBITMQ_SERVER_ADDITIONAL_ERL_ARGS environment variable are in place when using the official container images from Docker Hub.
RABBITMQ_SERVER_ADDITIONAL_ERL_ARGS: The RABBITMQ_SERVER_ADDITIONAL_ERL_ARGS environment variable sets additional parameters for the erl command used when invoking the RabbitMQ Server. The value of this variable is appended to the default list of arguments (RABBITMQ_SERVER_ERL_ARGS).


If you encounter a violation, ensure the RABBITMQ_SERVER_ADDITIONAL_ERL_ARGS environment variables is set.
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
