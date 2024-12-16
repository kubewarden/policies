# Affinity Pod Simple

This Policy sets podAffinity keys, regardless of the Pod Affinity scheduling type. List out the keys you want to check for. This Policy supports the following Pod Affinity types:
- preferredDuringSchedulingIgnoredDuringExecution 
- requiredDuringSchedulingIgnoredDuringExecution 


When working with pod affinity, this Policy will traverse through the entity and find the proper keys, regardless of pod affinity type. The follwing is an example of where the key is located. 
```
...
  spec:
    affinity:
      podAffinity:
        requiredDuringSchedulingIgnoredDuringExecution:
        - labelSelector:
            matchExpressions:
            - key: <keys>
```
https://kubernetes.io/docs/concepts/scheduling-eviction/assign-pod-node/
https://kubernetes.io/docs/reference/kubernetes-api/workload-resources/pod-v1/#PodAffinity


# Settings
```yaml
  settings:
    parameters:
      - name: keys
        type: array
        required: true
        value:
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

