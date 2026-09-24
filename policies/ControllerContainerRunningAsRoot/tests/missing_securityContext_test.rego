package policy

test_missing_securityContext_pod {
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
          "name": "no-security-context-demo",
        },
        "spec": {
          "containers": [
            {
              "name": "sec-ctx-demo",
              "image": "busybox",
              "command": [
                "sh",
                "-c",
                "sleep 1h"
              ],
            },
            {
              "name": "sec-ctx-demo2",
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

  # 1 violation per container, no securityContext defined anywhere
  count(violation) == 2 with input as testcase
}

test_missing_securityContext_deployment {
  testcase = {
    "parameters": {
      "exclude_namespaces": [],
      "exclude_label_key": "",
      "exclude_label_value": "",
    },
    "review": {
      "object": {
        "apiVersion": "apps/v1",
        "kind": "Deployment",
        "metadata": {
          "name": "no-security-context-demo",
        },
        "spec": {
          "template": {
            "spec": {
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
    }
  }

  count(violation) == 1 with input as testcase
}
