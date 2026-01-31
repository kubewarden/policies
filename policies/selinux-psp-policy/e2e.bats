#!/usr/bin/env bats

@test "RunAsAny: accept any pod" {
  run kwctl run annotated-policy.wasm \
    -r test_data/pod_plain.json \
    -s '{"rule": "RunAsAny"}'

  [ "$status" -eq 0 ]
  [[ "$output" == *"allowed\":true"* ]]
}

@test "MustRunAs: mutate pod with missing settings" {
  run kwctl run annotated-policy.wasm \
    -r test_data/pod_plain.json \
    -s '{"rule": "MustRunAs", "user": "system_u", "role": "system_r", "type": "spc_t", "level": "s0"}'

  [ "$status" -eq 0 ]
  [[ "$output" == *"allowed\":true"* ]]
  [[ "$output" == *"patchType\":\"JSONPatch"* ]]
}

@test "MustRunAs: reject pod with wrong settings" {
  run kwctl run annotated-policy.wasm \
    -r test_data/pod_bad_selinux.json \
    -s '{"rule": "MustRunAs", "user": "system_u", "role": "system_r", "type": "spc_t", "level": "s0"}'

  [ "$status" -eq 0 ]
  [[ "$output" == *"allowed\":false"* ]]
}