package policy

test_replicationcontroller_missing_security_context {
  testcase = {
    "parameters": {
      "exclude_namespaces": [],
      "exclude_label_key": "",
      "exclude_label_value": "",
    },
    "review": {
      "object": {
        "apiVersion": "v1",
        "kind": "ReplicationController",
        "metadata": {
          "name": "nginx",
        },
        "spec": {
          "replicas": 3,
          "selector": {"app": "nginx"},
          "template": {
            "metadata": {"labels": {"app": "nginx"}},
            "spec": {
              "containers": [
                {
                  "name": "nginx",
                  "image": "nginx:1.29",
                }
              ]
            }
          }
        }
      }
    }
  }

  count(violation) == 1 with input as testcase
}
