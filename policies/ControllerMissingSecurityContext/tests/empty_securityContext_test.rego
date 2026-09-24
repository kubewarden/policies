package policy

test_empty_pod_security_context {
  testcase = {
    "parameters": {
      "exclude_namespaces": [],
      "exclude_label_key": "",
      "exclude_label_value": "",
    },
    "review": {
      "object": {
        "apiVersion": "v1",
        "kind": "Pod",
        "metadata": {
          "name": "empty-pod-security-context",
        },
        "spec": {
          # the API server defaults this field to an empty object
          "securityContext": {},
          "containers": [
            {
              "name": "sec-ctx-demo",
              "image": "busybox",
            }
          ]
        }
      }
    }
  }

  count(violation) == 1 with input as testcase
}

test_empty_container_security_context {
  testcase = {
    "parameters": {
      "exclude_namespaces": [],
      "exclude_label_key": "",
      "exclude_label_value": "",
    },
    "review": {
      "object": {
        "apiVersion": "v1",
        "kind": "Pod",
        "metadata": {
          "name": "empty-container-security-context",
        },
        "spec": {
          "containers": [
            {
              "name": "sec-ctx-demo",
              "image": "busybox",
              "securityContext": {},
            }
          ]
        }
      }
    }
  }

  count(violation) == 1 with input as testcase
}
