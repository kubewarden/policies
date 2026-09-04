package main

import (
	"encoding/json"
	"testing"

	mapset "github.com/deckarep/golang-set/v2"
	kubewarden_protocol "github.com/kubewarden/policy-sdk-go/protocol"
)

func TestMutation(t *testing.T) {
	request := Request{
		User:     "tonio",
		Action:   "eats",
		Resource: "banana",
	}

	settings := Settings{
		ForbiddenResources: mapset.NewSet("banana", "apple"),
		DefaultResource:    "hay",
	}

	payload, err := json.Marshal(RawValidationRequest{
		Request:  request,
		Settings: settings,
	})
	if err != nil {
		t.Errorf("Unexpected error: %+v", err)
	}

	responsePayload, err := validate(payload)
	if err != nil {
		t.Errorf("Unexpected error: %+v", err)
	}

	var response kubewarden_protocol.ValidationResponse
	if err := json.Unmarshal(responsePayload, &response); err != nil {
		t.Errorf("Unexpected error: %+v", err)
	}

	if response.Accepted != true {
		t.Error("Unexpected rejection")
	}

	if response.MutatedObject == nil {
		t.Error("Unexpected mutation")
	}

	mutatedRequestJSON, err := json.Marshal(response.MutatedObject.(map[string]any))
	if err != nil {
		t.Errorf("Unexpected error: %+v", err)
	}

	if err := json.Unmarshal(mutatedRequestJSON, &request); err != nil {
		t.Errorf("Unexpected error: %+v", err)
	}

	if request.Resource != "hay" {
		t.Errorf("Unexpected mutation: %+v", response.MutatedObject)
	}
}

func TestAcceptWithoutMutation(t *testing.T) {
	request := Request{
		User:     "tonio",
		Action:   "eats",
		Resource: "spinach",
	}

	settings := Settings{
		ForbiddenResources: mapset.NewSet("banana", "apple"),
		DefaultResource:    "hay",
	}

	payload, err := json.Marshal(RawValidationRequest{
		Request:  request,
		Settings: settings,
	})
	if err != nil {
		t.Errorf("Unexpected error: %+v", err)
	}

	responsePayload, err := validate(payload)
	if err != nil {
		t.Errorf("Unexpected error: %+v", err)
	}

	var response kubewarden_protocol.ValidationResponse
	if err := json.Unmarshal(responsePayload, &response); err != nil {
		t.Errorf("Unexpected error: %+v", err)
	}

	if !response.Accepted {
		t.Error("Unexpected rejection")
	}

	if response.MutatedObject != nil {
		t.Error("Unexpected mutation")
	}
}

func TestRejectInvalidPayload(t *testing.T) {
	payload := []byte(`{"foo": "bar"}`)

	responsePayload, err := validate(payload)
	if err != nil {
		t.Errorf("Unexpected error: %+v", err)
	}

	var response kubewarden_protocol.ValidationResponse
	if err := json.Unmarshal(responsePayload, &response); err != nil {
		t.Errorf("Unexpected error: %+v", err)
	}

	if response.Accepted {
		t.Error("Unexpected acceptance")
	}

	if *response.Code != 400 {
		t.Errorf("Unexpected status code")
	}
}
