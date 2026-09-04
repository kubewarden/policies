package main

import (
	"encoding/json"
	"testing"

	mapset "github.com/deckarep/golang-set/v2"
	kubewarden_protocol "github.com/kubewarden/policy-sdk-go/protocol"
	kubewarden_testing "github.com/kubewarden/policy-sdk-go/testing"
)

func TestApproval(t *testing.T) {
	for _, tcase := range []struct {
		name     string
		testData string
		settings Settings
	}{
		{
			name:     "empty settings allows safe sysctls",
			testData: "test_data/request-pod-safe-sysctls.json",
			settings: Settings{},
		},
		{
			name:     "pod without sysctl always allowed",
			testData: "test_data/request-pod-no-sysctl.json",
			settings: Settings{},
		},
		{
			name:     "pod with allowedUnsafe sysctl",
			testData: "test_data/request-pod-somaxconn.json",
			settings: Settings{
				AllowedUnsafeSysctls: mapset.NewThreadUnsafeSet("net.core.somaxconn", "bar"),
				ForbiddenSysctls:     mapset.NewThreadUnsafeSet("net.*"),
			},
		},
	} {
		payload, err := kubewarden_testing.BuildValidationRequestFromFixture(
			tcase.testData,
			&tcase.settings)
		if err != nil {
			t.Errorf("on test %q, got unexpected error '%+v'", tcase.name, err)
		}

		responsePayload, err := validate(payload)
		if err != nil {
			t.Errorf("on test %q, got unexpected error '%+v'", tcase.name, err)
		}

		var response kubewarden_protocol.ValidationResponse
		if err := json.Unmarshal(responsePayload, &response); err != nil {
			t.Errorf("on test %q, got unexpected error '%+v'", tcase.name, err)
		}

		if response.Accepted != true {
			t.Errorf("on test %q, got unexpected rejection", tcase.name)
		}
	}
}

func TestRejection(t *testing.T) {
	for _, tcase := range []struct {
		name     string
		testData string
		settings Settings
		error    string
	}{
		{
			name:     "empty settings reject non safe sysctls",
			testData: "test_data/request-pod-somaxconn.json",
			settings: Settings{},
			error:    "sysctl net.core.somaxconn is not on safe list, nor is in the allowedUnsafeSysctls list",
		},
		{
			name:     "all sysctls forbidden",
			testData: "test_data/request-pod-somaxconn.json",
			settings: Settings{
				AllowedUnsafeSysctls: mapset.NewThreadUnsafeSet[string](),
				ForbiddenSysctls:     mapset.NewThreadUnsafeSet("*"),
			},
			error: "sysctl net.core.somaxconn is on the forbidden list",
		},
		{
			name:     "net.* sysctls forbidden",
			testData: "test_data/request-pod-somaxconn.json",
			settings: Settings{
				AllowedUnsafeSysctls: mapset.NewThreadUnsafeSet[string](),
				ForbiddenSysctls:     mapset.NewThreadUnsafeSet("net.*"),
			},
			error: "sysctl net.core.somaxconn is on the forbidden list",
		},
	} {
		payload, err := kubewarden_testing.BuildValidationRequestFromFixture(
			tcase.testData,
			&tcase.settings)
		if err != nil {
			t.Errorf("on test %q, got unexpected error '%+v'", tcase.name, err)
		}

		responsePayload, err := validate(payload)
		if err != nil {
			t.Errorf("on test %q, got unexpected error '%+v'", tcase.name, err)
		}

		var response kubewarden_protocol.ValidationResponse
		if err := json.Unmarshal(responsePayload, &response); err != nil {
			t.Errorf("on test %q, got unexpected error '%+v'", tcase.name, err)
		}

		if response.Accepted != false {
			t.Errorf("on test %q, got unexpected approval", tcase.name)
		}

		if *response.Message != tcase.error {
			t.Errorf("on test %q, got '%s' instead of '%s'",
				tcase.name, *response.Message, tcase.error)
		}
	}
}
