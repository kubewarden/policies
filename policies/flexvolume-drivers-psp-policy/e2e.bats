#!/usr/bin/env bats

@test "Accept pod with allowed flexVolume driver" {
  run kwctl run policy.wasm \
    -r test_data/request_allowed.json \
    --settings-json '{"allowedFlexVolumes": [{"driver": "example/allowed-driver"}]}'

  [ "$status" -eq 0 ]
  [[ "$output" == *"allowed\":true"* ]]
}

@test "Reject pod with unlisted flexVolume driver" {
  run kwctl run policy.wasm \
    -r test_data/request_allowed.json \
    --settings-json '{"allowedFlexVolumes": [{"driver": "vendor/unknown-driver"}]}'

  [ "$status" -eq 0 ]
  [[ "$output" == *"allowed\":false"* ]]
}