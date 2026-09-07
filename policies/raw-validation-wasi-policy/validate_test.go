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
			Resource: "hay",
		},
		Settings: Settings{
			ValidUsers:     mapset.NewSet[string]("tonio", "wanda"),
			ValidActions:   mapset.NewSet[string]("eats", "likes"),
			ValidResources: mapset.NewSet[string]("hay", "carrot", "banana"),
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
}

func TestValidateRequestReject(t *testing.T) {
	validationRequest := RawValidationRequest{
		Request: Request{
			User:     "oscar",
			Action:   "eats",
			Resource: "hay",
		},
		Settings: Settings{
			ValidUsers:     mapset.NewSet[string]("tonio", "wanda"),
			ValidActions:   mapset.NewSet[string]("eats", "likes"),
			ValidResources: mapset.NewSet[string]("hay", "carrot", "banana"),
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

	if response.Accepted {
		t.Errorf("response should be rejected")
	}

	if *response.Message != "User 'oscar' is not allowed" {
		t.Errorf("wrong message: %s", *response.Message)
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

func TestValidateRequestWithNullSetDoesNotPanic(t *testing.T) {
	payload := []byte(`{
		"request":{"user":"tonio","action":"eats","resource":"hay"},
		"settings":{"validUsers":null,"validActions":["eats"],"validResources":["hay"]}
	}`)
	responseJSON := validate(payload)

	var response ValidationResponse

	err := json.Unmarshal(responseJSON, &response)
	if err != nil {
		t.Fatalf("cannot unmarshal response: %v", err)
	}

	if response.Accepted {
		t.Error("request should be rejected because validUsers is empty")
	}
}
