# RabbitMQ Enforce Environment Variable - RABBITMQ_VM_MEMORY_HIGH_WATERMARK

This Policy ensures RABBITMQ_VM_MEMORY_HIGH_WATERMARK environment variable are in place when using the official container images from Docker Hub.
RABBITMQ_VM_MEMORY_HIGH_WATERMARK: The RABBITMQ_VM_MEMORY_HIGH_WATERMARK environment variable sets the memory threshold at which the flow control is triggered. Can be absolute or relative to the amount of RAM available to the OS.


If you encounter a violation, ensure the RABBITMQ_VM_MEMORY_HIGH_WATERMARK environment variables is set.
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
