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
  snapshot_class: "snapshot_class"
  pvc_name: "pvc_name"
  exclude_label_key: "" # optional
  exclude_label_value: "" # optional
```

# Resources

Policy applies to resources kinds:
`VolumeSnapshot`
