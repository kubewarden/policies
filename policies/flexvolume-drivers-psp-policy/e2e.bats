#!/usr/bin/env bats

@test "Accept pod with allowed flexVolume driver" {
  run kwctl run annotated-policy.wasm \
    -r test_data/pod_with_flexvolume.json \
    --settings-json '{"allowedFlexVolumes":[{"driver":"example/allowed-driver"}]}'

  [ "$status" -eq 0 ]
  [[ "$output" == *"\"allowed\":true"* ]]
}

@test "Accept pod without any flexVolume" {
  run kwctl run annotated-policy.wasm \
    -r test_data/pod_creation.json \
    --settings-json '{"allowedFlexVolumes":[{"driver":"example/allowed-driver"}]}'

  [ "$status" -eq 0 ]
  [[ "$output" == *"\"allowed\":true"* ]]
}

@test "Reject pod with unlisted flexVolume driver" {
  run kwctl run annotated-policy.wasm \
    -r test_data/pod_with_flexvolume.json \
    --settings-json '{"allowedFlexVolumes":[{"driver":"vendor/unknown-driver"}]}'

  [ "$status" -eq 0 ]
  [[ "$output" == *"\"allowed\":false"* ]]
}