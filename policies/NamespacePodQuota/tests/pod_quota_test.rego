package policy

test_key_value_exists {
	testcase = {
        "parameters": {
            "pod_quota": "3",
            "namespace": "magalix"
        },
        "review": {
            "object": {
                "apiVersion": "v1",
                "kind": "ResourceQuota",
                "metadata": {
                    "name": "pod-demo",
                    "namespace": "magalix"
                },
                "spec": {
                    "hard": {
                        "pods": "2"
                    }
                }                
            }
        }
    }

	count(violation) == 0 with input as testcase
}

test_pod_quota_too_high {
	testcase = {
        "parameters": {
            "pod_quota": "3",
            "namespace": "magalix"
        },
        "review": {
            "object": {
                "apiVersion": "v1",
                "kind": "ResourceQuota",
                "metadata": {
                    "name": "pod-demo",
                    "namespace": "magalix"
                },
                "spec": {
                    "hard": {
                        "pods": "9"
                    }
                }
            }
        }
    }

	count(violation) == 1 with input as testcase
}
