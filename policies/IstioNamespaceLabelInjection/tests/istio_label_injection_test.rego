package policy

test_key_value_exists {
	testcase = {
        "parameters": {
            "namespaces": [
                "namespace",
            ],
        },
        "review": {
            "object": {
                "apiVersion": "v1",
                "kind": "Namespace",
                "metadata": {
                    "name": "namespace",
                    "labels": {
                        "istio-injection": "enabled"
                    }
                }
            }
        }
    }
	count(violation) == 0 with input as testcase
}

test_label_missing {
	testcase = {
        "parameters": {
            "namespaces": [
                "namespace",
            ],
        },
        "review": {
            "object": {
                "apiVersion": "v1",
                "kind": "Namespace",
                "metadata": {
                    "name": "namespace"
                }
            }
        }
    }

	count(violation) == 1 with input as testcase
}
