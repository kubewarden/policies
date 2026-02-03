#!/usr/bin/env bats

@test "RunAsAny: accept any pod" {
  echo '{"rule": "RunAsAny"}' > settings.json

  run kwctl run annotated-policy.wasm \
    -r test_data/pod_plain.json \
    -s settings.json

  # Debug logs
  echo "Status: $status"
  echo "Output: $output"

  [ "$status" -eq 0 ]
  [[ "$output" == *"allowed\":true"* ]]
}

@test "MustRunAs: mutate pod with missing settings" {
  echo '{"rule": "MustRunAs", "user": "system_u", "role": "system_r", "type": "spc_t", "level": "s0"}' > settings.json

  run kwctl run annotated-policy.wasm \
    -r test_data/pod_plain.json \
    -s settings.json

  echo "Status: $status"
  echo "Output: $output"

  [ "$status" -eq 0 ]
  [[ "$output" == *"allowed\":true"* ]]
  [[ "$output" == *"patchType\":\"JSONPatch"* ]]
}

@test "MustRunAs: reject pod with wrong settings" {
  echo '{"rule": "MustRunAs", "user": "system_u", "role": "system_r", "type": "spc_t", "level": "s0"}' > settings.json

  run kwctl run annotated-policy.wasm \
    -r test_data/pod_bad_selinux.json \
    -s settings.json

  echo "Status: $status"
  echo "Output: $output"

  [ "$status" -eq 0 ]
  [[ "$output" == *"allowed\":false"* ]]
}