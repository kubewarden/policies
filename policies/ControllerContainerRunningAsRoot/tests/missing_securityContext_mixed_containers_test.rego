package policy

test_missing_securityContext_mixed_containers {
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
              "name": "sec-ctx-demo-missing",
              "image": "busybox",
              "command": [
                "sh",
                "-c",
                "sleep 1h"
              ],
            },
            {
              "securityContext" : {
                "runAsNonRoot": true,
              },
              "name": "sec-ctx-demo-ok",
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

  # pod-level securityContext is entirely absent; one container omits
  # securityContext entirely (violation), the sibling container explicitly
  # sets runAsNonRoot: true (no violation) -> only 1 violation total
  count(violation) == 1 with input as testcase
}
