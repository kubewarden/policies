# Containers Block Ports Range

This Policy checks for container ports that are set below the set value. TCP ports under 1024 are reserved so we recommend setting your Policy to 1024 or higher. 


Choose ports over the value that is specified in the Policy. 
```
...
  spec:
    containers:
      - ports:
        - containerPort: <target_port>
```
https://www.iana.org/assignments/service-names-port-numbers/service-names-port-numbers.txt


# Settings
```yaml
  settings:
    parameters:
      - name: target_port
        type: integer
        required: true
        value: 1024
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
