# Prohibit Creating Namespace Starting With Prefix

Using this Policy, you can prohibit certain namespaces from containing a specified combination of letters and/or numbers. 


Specify a `namespace` that is something other than what is listed in the Policy. 
```
metadata:
  name: <namespace_name>
```
https://kubernetes.io/docs/concepts/overview/working-with-objects/namespaces/#working-with-namespaces


# Settings
```yaml
  settings:
    parameters:
      - name: namespace_name
        type: string
        required: true
        value: kube-
```

# Resources
Policy applies to resources kinds:
`Namespace`
