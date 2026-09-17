#!/usr/bin/env bats

# Release flow check: exercises release-please for a Rego/gatekeeper policy.

@test "accept because Ingress host is unique" {
  run kwctl run annotated-policy.wasm \
              -r ./test_data/ingress.json \
              --allow-context-aware \
              --replay-host-capabilities-interactions ./test_data/k8s_ctx_no_duplicates.yml
  # this prints the output when one the checks below fails
  echo "output = ${output}"

  # request accepted
  [ "$status" -eq 0 ]
  [ $(expr "$output" : '.*allowed.*true') -ne 0 ]
}

@test "reject because Ingress host is not unique" {
  run kwctl run annotated-policy.wasm \
              -r ./test_data/ingress.json \
              --allow-context-aware \
              --replay-host-capabilities-interactions ./test_data/k8s_ctx_duplicate.yml

  # this prints the output when one the checks below fails
  echo "output = ${output}"

  # request rejected
  [ "$status" -eq 0 ]
  [ $(expr "$output" : '.*allowed.*false') -ne 0 ]
  [ $(expr "$output" : '.*ingress host conflicts with an existing ingress <foo.bar.com>.*') -ne 0 ]
}
