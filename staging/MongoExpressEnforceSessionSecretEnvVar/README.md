# Mongo-Express Enforce Environment Variable - ME_CONFIG_SITE_SESSIONSECRET

This Policy ensures ME_CONFIG_SITE_SESSIONSECRET environment variable are in place when using the official container images from Docker Hub.
ME_CONFIG_SITE_SESSIONSECRET: The ME_CONFIG_SITE_SESSIONSECRET environment variable is used to sign the session ID cookie by [express-session middleware](https://www.npmjs.com/package/express-session).


If you encounter a violation, ensure the ME_CONFIG_SITE_SESSIONSECRET environment variables is set.
For futher information about the Mongo-Express Docker container, check here: https://hub.docker.com/_/mongo-express


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
