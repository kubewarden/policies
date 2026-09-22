# MYSQL Enforce Environment Variable - MYSQL_ONETIME_PASSWORD

This Policy ensures MYSQL_ONETIME_PASSWORD environment variable are in place when using the official container images from Docker Hub.
MYSQL_ONETIME_PASSWORD: The MYSQL_ONETIME_PASSWORD environment variable is set, the root user's password is set as expired and must be changed before MySQL can be used normally 


If you encounter a violation, ensure the MYSQL_ONETIME_PASSWORD environment variables is set.
For futher information about the MYSQL Docker container, check here: https://hub.docker.com/_/mysql


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
