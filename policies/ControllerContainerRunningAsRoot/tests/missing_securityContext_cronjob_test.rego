package policy

test_missing_securityContext_cronjob {
  testcase = {
    "parameters": {
      "exclude_namespaces": [],
      "exclude_label_key": "",
      "exclude_label_value": "",
    },
    "review": {
      "object": {
        "apiVersion": "batch/v1",
        "kind": "CronJob",
        "metadata": {
          "name": "no-security-context-demo",
        },
        "spec": {
          "jobTemplate": {
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
    }
  }

  # CronJob traversal path (spec.jobTemplate.spec.template.spec), both
  # pod-level and container-level securityContext entirely omitted
  count(violation) == 1 with input as testcase
}
