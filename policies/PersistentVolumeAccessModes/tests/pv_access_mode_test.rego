package policy

test_key_value_exists {
	testcase = {
		"parameters": {
			"name": "test-volume",
			"access_mode": "ReadWriteOnce",
		},
		"review": {
            "object": {
                "apiVersion": "v1",
                "kind": "PersistentVolume",
                "metadata": {
                    "name": "test-volume"
                },
                "spec": {
                    "accessModes": [
                        "ReadWriteOnce"
                    ],
                    "capacity": {
                        "storage": "200Gi"
                    },
                    "gcePersistentDisk": {
                        "fsType": "ext4",
                        "pdName": "my-data-disk"
                    },
                    "storageClassName": "gcp-disk"
                } 
            }
	    }
    }
	count(violation) == 0 with input as testcase
}

test_access_mode_missing {
	testcase = {
		"parameters": {
			"name": "test-volume",
			"access_mode": "ReadWriteOnce",
		},
		"review": {
            "object": {
                "apiVersion": "v1",
                "kind": "PersistentVolume",
                "metadata": {
                    "name": "test-volume"
                },
                "spec": {
                    "accessModes": [
                        "ReadOnlyMany"
                    ],
                    "capacity": {
                        "storage": "200Gi"
                    },
                    "storageClassName": "gcp-disk"
                }
            }
	    }
    }

	count(violation) == 1 with input as testcase
}
