# Affinity Node Simple

This Policy sets NodeAffinity keys, regardless of the Node Affinity selector or scheduling type. List out the keys you want to check to ensure you are using the right keys. This Policy supports the following Node Affinity types:
- preferredDuringSchedulingIgnoredDuringExecution
- requiredDuringSchedulingIgnoredDuringExecution


When working with Node Affinity, this policy will traverse through the entity and find the proper keys, regardless of affinity type. The follwing is an example of where the key is located. 

```
...
  spec:
    affinity:
      nodeAffinity:
        requiredDuringSchedulingIgnoredDuringExecution:
          nodeSelectorTerms:
          - matchExpressions:
            - key: <keys>
```

https://kubernetes.io/docs/concepts/scheduling-eviction/assign-pod-node/


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

