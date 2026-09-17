#!/usr/bin/env bats

# Release flow check: exercises release-please for a Rego/opa policy.

@test "accept because not a NodePort service" {
  run kwctl run -e opa annotated-policy.wasm -r test_data/service-clusterip.json

  # this prints the output when one the checks below fails
  echo "output = ${output}"

  # request accepted
  [ "$status" -eq 0 ]
  [ $(expr "$output" : '.*allowed.*true') -ne 0 ]
}

@test "reject because NodePort services are not allowed" {
  run kwctl run -e opa annotated-policy.wasm -r test_data/service_nodeport.json

  # this prints the output when one the checks below fails
  echo "output = ${output}"

  # request rejected
  [ "$status" -eq 0 ]
  [ $(expr "$output" : '.*allowed.*false') -ne 0 ]
  [ $(expr "$output" : '.*Service of type NodePort are not allowed.*') -ne 0 ]
}
