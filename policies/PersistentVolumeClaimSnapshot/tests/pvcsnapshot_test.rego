package policy

test_key_value_exists {
	testcase = {
		"parameters": {
			"snapshot_class": "csi-hostpath-snapclass",
			"pvc_name": "pvc-test",
			"exclude_namespace": "",
			"exclude_label_key": "",
			"exclude_label_value": "",
		},
		"review": {
            "object": {
                "apiVersion": "snapshot.storage.k8s.io/v1",
                "kind": "VolumeSnapshot",
                "metadata": {
                    "name": "new-snapshot-test"
                },
                "spec": {
                    "volumeSnapshotClassName": "csi-hostpath-snapclass",
                    "source": {
                        "persistentVolumeClaimName": "pvc-test"
                    }
                } 
            }
	    }
    }
	count(violation) == 0 with input as testcase
}

test_snapshot_class_wrong {
	testcase = {
		"parameters": {
			"snapshot_class": "csi-hostpath-snapclass",
			"pvc_name": "pvc-test",
			"exclude_label_key": "",
			"exclude_label_value": "",
		},
		"review": {
            "object": {
                "apiVersion": "snapshot.storage.k8s.io/v1",
                "kind": "VolumeSnapshot",
                "metadata": {
                    "name": "new-snapshot-test"
                },
                "spec": {
                    "volumeSnapshotClassName": "another-snapclass",
                    "source": {
                        "persistentVolumeClaimName": "pvc-test"
                    }
                }
            }
	    }
    }

	count(violation) == 1 with input as testcase
}
