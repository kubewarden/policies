# Network Allow Egress Dns To Coredns

If you are using a CNI that allows for Network Policies, you can use this Policy to allow egress traffic from a specified namespace, to your coredns pods with the label `k8s-app: kube-dns`. The Policy specifically looks for port UDP 53. 

By default, if no policies exist in a namespace, then all ingress and egress traffic is allowed to and from pods in that namespace. 


You must specifiy a source namespace to allow Egress to coredns on UDP53. 

```
...
metadata:
  namespace: <namespace>
...
```
and
```
...
egress:
- to:
  - podSelector:
      matchLabels:
        k8s-app: kube-dns
...
```

https://kubernetes.io/docs/concepts/services-networking/network-policies/#targeting-a-namespace-by-its-name


# Settings
```yaml
  settings:
    parameters:
      - name: namespace
        type: string
        required: true
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

