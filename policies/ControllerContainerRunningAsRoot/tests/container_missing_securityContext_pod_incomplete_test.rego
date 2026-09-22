package policy

test_container_missing_securityContext_pod_incomplete {
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
          "securityContext" : {
            "runAsUser": 1000,
          },
          "containers": [
            {
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

  # container omits securityContext entirely, pod-level securityContext is
  # present but does not set runAsNonRoot -> still a violation
  count(violation) == 1 with input as testcase
}
