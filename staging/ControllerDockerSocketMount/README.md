# Containers Mounting Docker Socket

This Policy checks the mounting of the `docker.sock` socket of the node into the container. The docker socket filename can be altered based on your configuration. The full path is unnecessary as the Policy accounts for any path.  


Ensure workloads aren't mounting the `docker.sock` (or other configured socket name) in order to avoid a violation. 
```
...
  spec:
    template:
      spec:
        volumes:
          - name: docker-sock
            hostPath:
              path: "<path>/<this value>"
              type: File
```


# Settings
```yaml
  settings:
    parameters:
      - name: docker_sock
        type: string
        required: true
        value: docker.sock
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
