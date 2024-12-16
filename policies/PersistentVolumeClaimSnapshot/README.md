# Persistent Volume Claim Snapshot

This Policy checks for a PVC Snapshot. 


When using Persistent Volume Claim snapshots, ensure both the `snapshot_class` and `pvc_name` define match what's in the Policy. 
```
spec:
  volumeSnapshotClassName: <snapshot_class>
  source:
    persistentVolumeClaimName: <pvc_name>
```


# Settings
```yaml
  settings:
    parameters:
      - name: snapshot_class
        type: string
        required: true
        value:
      - name: pvc_name
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

