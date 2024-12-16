# Persistent Volume Access Modes

A PersistentVolume can be mounted on a host in any way supported by the resource provider. As shown in the table below, providers will have different capabilities and each PV's access modes are set to the specific modes supported by that particular volume. For example, NFS can support multiple read/write clients, but a specific NFS PV might be exported on the server as read-only. Each PV gets its own set of access modes describing that specific PV's capabilities.

The access modes are:

* ReadWriteOnce
* ReadOnlyMany
* ReadWriteMany
* ReadWriteOncePod


Ensure the <name> of your Persistent Volume and <access_mode> set in your Policy match the PV you want to check.
```
apiVersion: v1
kind: PersistentVolume
metadata: 
  name: <name>
spec: 
  accessModes: 
    - <access_mode>
```


# Settings
```yaml
  settings:
    parameters:
      - name: name
        type: string
        required: true
        value:
      - name: access_mode
        type: string
        required: true
        value:
```

# Resources
Policy applies to resources kinds:

