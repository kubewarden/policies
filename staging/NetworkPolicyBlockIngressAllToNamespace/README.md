# Network Block All Ingress Traffic To Namespace

If you are using a CNI that allows for Network Policies, you can use this Policy to block all Ingress traffic from a specified namespace. 

By default, if no policies exist in a namespace, then all ingress and egress traffic is allowed to and from pods in that namespace. 


Validate your use case and check network policies for traffic blocking. 

https://kubernetes.io/docs/concepts/services-networking/network-policies/


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

