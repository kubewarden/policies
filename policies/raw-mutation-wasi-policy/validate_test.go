package main

import (
	"encoding/json"
	"testing"

	mapset "github.com/deckarep/golang-set/v2"
)

func TestValidateRequestAccept(t *testing.T) {
	validationRequest := RawValidationRequest{
		Request: Request{
			User:     "tonio",
			Action:   "eats",
			Resource: "banana",
		},
		Settings: Settings{
			ForbiddenResources: mapset.NewSet[string]("carrot", "banana"),
			DefaultResource:    defaultResourceHay,
		},
	}

	validationRequestJSON, err := json.Marshal(&validationRequest)
	if err != nil {
		t.Errorf("unexpected error: %v", err)
	}

	responseJSON := validate(validationRequestJSON)

	var response ValidationResponse
	err = json.Unmarshal(responseJSON, &response)
	if err != nil {
		t.Errorf("unexpected error: %v", err)
	}

	if !response.Accepted {
		t.Errorf("response should be accepted: %s", *response.Message)
	}

	if response.MutatedObject.Resource != defaultResourceHay {
		t.Errorf("response should be mutated")
	}
}

func TestValidateAcceptWithoutMutating(t *testing.T) {
	validationRequest := RawValidationRequest{
		Request: Request{
			User:     "tonio",
			Action:   "eats",
			Resource: "spinach",
		},
		Settings: Settings{
			ForbiddenResources: mapset.NewSet[string]("carrot", "banana"),
			DefaultResource:    defaultResourceHay,
		},
	}

	validationRequestJSON, err := json.Marshal(&validationRequest)
	if err != nil {
		t.Errorf("unexpected error: %v", err)
	}

	responseJSON := validate(validationRequestJSON)

	var response ValidationResponse
	err = json.Unmarshal(responseJSON, &response)
	if err != nil {
		t.Errorf("unexpected error: %v", err)
	}

	if !response.Accepted {
		t.Errorf("response should be accepted: %s", *response.Message)
	}

	if response.MutatedObject != nil {
		t.Errorf("response should not be mutated")
	}
}

func TestValidateSettingsRejectInvalidPayload(t *testing.T) {
	payload := []byte(`{"foo": "bar"}`)

	responseJSON := validate(payload)

	var response ValidationResponse
	err := json.Unmarshal(responseJSON, &response)
	if err != nil {
		t.Errorf("unexpected error: %v", err)
	}

	if response.Accepted {
		t.Errorf("response should be rejected")
	}

	if *response.Message != "Error deserializing validation request: json: unknown field \"foo\"" {
		t.Errorf("wrong message: %s", *response.Message)
	}
}
