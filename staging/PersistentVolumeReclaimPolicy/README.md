# Persistent Volume Reclaim Policy Should Be Set To Retain

This Policy checks to see whether or not the persistent volume reclaim policy is set.

PersistentVolumes can have various reclaim policies, including "Retain", "Recycle", and "Delete". For dynamically provisioned PersistentVolumes, the default reclaim policy is "Delete". This means that a dynamically provisioned volume is automatically deleted when a user deletes the corresponding PersistentVolumeClaim. This automatic behavior might be inappropriate if the volume contains precious data. In that case, it is more appropriate to use the "Retain" policy. With the "Retain" policy, if a user deletes a PersistentVolumeClaim, the corresponding PersistentVolume is not be deleted. Instead, it is moved to the Released phase, where all of its data can be manually recovered.


Check your reclaim policy configuration within your PersistentVolume configuration. 
```
spec:
  persistentVolumeReclaimPolicy: <pv_policy>
```

https://kubernetes.io/docs/tasks/administer-cluster/change-pv-reclaim-policy/#why-change-reclaim-policy-of-a-persistentvolume


# Settings
```yaml
  settings:
    parameters:
      - name: pv_policy
        type: string
        required: true
        value: Retain
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
`PersistentVolume`
