# Antiaffinity Pod Simple

This Policy sets antiPodAffinity keys, regardless of the Anti Pod Affinity scheduling type. List out the keys you want to check for. This Policy supports the following Pod Anti-Affinity types:
- preferredDuringSchedulingIgnoredDuringExecution 
- requiredDuringSchedulingIgnoredDuringExecution 

There is 1 parameter for this Policy:
- keys (array) : The policy will check for `key` you are basing the anti-affinty on. 


When working with Pod Anti-Affinity, this Policy will traverse through the entity and find the proper keys, regardless of pod affinity type. The follwing is an example of where the key is located. 
```
...
  spec:
    podAntiAffinity:
      preferredDuringSchedulingIgnoredDuringExecution:
      - podAffinityTerm:
          labelSelector:
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

