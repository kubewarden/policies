# Container Prohibit Image Tag

Prohibit certain image tags by specifying them in the Policy. The Policy will also violate if the a tag is not set, or is set to `latest`. 

Note: You should avoid using the :latest tag when deploying containers in production as it is harder to track which version of the image is running and more difficult to roll back properly.


Configure an image tag that is not in the Policy. 
```
...
  spec:
    containers:
    - image: registry/image_name:<tag>
```
https://kubernetes.io/docs/concepts/configuration/overview/#container-images


# Settings
```yaml
  settings:
    parameters:
      - name: image_tag
        type: string
        required: true
        value: latest
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
