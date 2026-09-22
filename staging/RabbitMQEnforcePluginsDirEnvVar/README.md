# RabbitMQ Enforce Environment Variable - RABBITMQ_PLUGINS_DIR

This Policy ensures RABBITMQ_PLUGINS_DIR environment variable are in place when using the official container images from Docker Hub.
RABBITMQ_PLUGINS_DIR: The list of directories where plugin archive files are located and extracted from. This is PATH-like variable, where different paths are separated by an OS-specific separator (: for Unix, ; for Windows). Plugins can be installed to any of the directories listed here.


If you encounter a violation, ensure the RABBITMQ_PLUGINS_DIR environment variables is set.
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
