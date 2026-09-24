package policy

test_pod_missing_securityContext_container_runAsNonRoot_true {
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
          "name": "security-context-demo",
        },
        "spec": {
          "containers": [
            {
              "securityContext" : {
                "runAsNonRoot": true,
              },
              "name": "sec-ctx-demo",
              "image": "busybox",
              "command": [
                "sh",
                "-c",
                "sleep 1h"
              ],
            }
          ]
        }
      }
    }
  }

  # pod-level securityContext is entirely absent, but the container
  # explicitly sets runAsNonRoot: true -> no violation
  count(violation) == 0 with input as testcase
}
