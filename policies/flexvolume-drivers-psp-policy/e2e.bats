#!/usr/bin/env bats

@test "Accept pod with an allowed flexVolume driver" {
  run kwctl run annotated-policy.wasm \
    -r test_data/pod_with_flexvolume.json \
    --settings-json '{"allowedFlexVolumes":[{"driver":"example/allowed-driver"}]}'

  [ "$status" -eq 0 ]
  [[ "$output" == *"\"allowed\":true"* ]]
}

@test "Accept pod without any flexVolume" {
  run kwctl run annotated-policy.wasm \
    -r test_data/pod_without_flexvolume.json \
    --settings-json '{"allowedFlexVolumes":[{"driver":"example/allowed-driver"}]}'

  [ "$status" -eq 0 ]
  [[ "$output" == *"\"allowed\":true"* ]]
}

@test "Reject pod with not listed flexVolume driver" {
  run kwctl run annotated-policy.wasm \
    -r test_data/pod_with_flexvolume.json \
    --settings-json '{"allowedFlexVolumes":[{"driver":"vendor/unknown-driver"}]}'

  [ "$status" -eq 0 ]
  [[ "$output" == *"\"allowed\":false"* ]]
}
