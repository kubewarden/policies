#!/usr/bin/env bats

@test "Accept a Deployment without policies specified" {
	run kwctl run --allow-context-aware \
		--raw -r test_data/deployment-no-policy.json \
		--replay-host-capabilities-interactions test_data/replay-session-with-workload-policy.yml \
		annotated-policy.wasm
	[ "$status" -eq 0 ]
	echo "$output"
	[ $(expr "$output" : '.*"allowed":true.*') -ne 0 ]
}

@test "Accept a Deployment with a policy present" {
	run kwctl run --allow-context-aware \
		--raw -r test_data/deployment-accepted.json \
		--replay-host-capabilities-interactions test_data/replay-session-with-workload-policy.yml \
		annotated-policy.wasm
	[ "$status" -eq 0 ]
	echo "$output"
	[ $(expr "$output" : '.*"allowed":true.*') -ne 0 ]
}

@test "Reject a Deployment referencing a non-existing WorkloadPolicy" {
	run kwctl run --allow-context-aware \
		--raw -r test_data/deployment-rejected.json \
		--replay-host-capabilities-interactions test_data/replay-session-no-workload-policy.yml \
		annotated-policy.wasm
	[ "$status" -eq 0 ]
	echo "$output"
	[ $(expr "$output" : '.*"allowed":false.*') -ne 0 ]
	[ $(expr "$output" : '.*is not found.*') -ne 0 ]
	[ $(expr "$output" : '.*"code":403.*') -ne 0 ]
}

@test "Reject a Deployment without policies specified when requireProtection is true" {
	run kwctl run --allow-context-aware \
		--raw -r test_data/deployment-no-policy.json \
		--replay-host-capabilities-interactions test_data/replay-session-with-workload-policy.yml \
		--settings-json '{"requireProtection": true}' \
		annotated-policy.wasm
	[ "$status" -eq 0 ]
	echo "$output"
	[ $(expr "$output" : '.*"allowed":false.*') -ne 0 ]
	[ $(expr "$output" : '.*is not protected by Runtime Enforcer.*') -ne 0 ]
}

@test "Accept a Deployment with a policy present when requireProtection is true" {
	run kwctl run --allow-context-aware \
		--raw -r test_data/deployment-accepted.json \
		--replay-host-capabilities-interactions test_data/replay-session-with-workload-policy.yml \
		--settings-json '{"requireProtection": true}' \
		annotated-policy.wasm
	[ "$status" -eq 0 ]
	echo "$output"
	[ $(expr "$output" : '.*"allowed":true.*') -ne 0 ]
}

@test "Reject a Deployment referencing a non-existing WorkloadPolicy when requireProtection is true" {
	run kwctl run --allow-context-aware \
		--raw -r test_data/deployment-rejected.json \
		--replay-host-capabilities-interactions test_data/replay-session-no-workload-policy.yml \
		--settings-json '{"requireProtection": true}' \
		annotated-policy.wasm
	[ "$status" -eq 0 ]
	echo "$output"
	[ $(expr "$output" : '.*"allowed":false.*') -ne 0 ]
	[ $(expr "$output" : '.*is not found.*') -ne 0 ]
	[ $(expr "$output" : '.*"code":403.*') -ne 0 ]
}

@test "Accept a Deployment without policies specified when requireProtection is false" {
	run kwctl run --allow-context-aware \
		--raw -r test_data/deployment-no-policy.json \
		--replay-host-capabilities-interactions test_data/replay-session-with-workload-policy.yml \
		--settings-json '{"requireProtection": false}' \
		annotated-policy.wasm
	[ "$status" -eq 0 ]
	echo "$output"
	[ $(expr "$output" : '.*"allowed":true.*') -ne 0 ]
}
