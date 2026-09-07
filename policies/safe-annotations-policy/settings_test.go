package main

import (
	"testing"

	"encoding/json"

	kubewarden_protocol "github.com/kubewarden/policy-sdk-go/protocol"
)

func TestParseValidSettings(t *testing.T) {
	settingsJSON := []byte(`
	{
		"denied_annotations": [ "foo", "bar" ],
		"mandatory_annotations": ["owner"],
		"constrained_annotations": {
			"cost-center": "cc-\\d+"
		}
	}`)

	settings := Settings{}
	err := json.Unmarshal(settingsJSON, &settings)
	if err != nil {
		t.Errorf("Unexpected error %+v", err)
	}

	expectedDeniedAnnotations := []string{"foo", "bar"}
	for _, exp := range expectedDeniedAnnotations {
		if !settings.DeniedAnnotations.Contains(exp) {
			t.Errorf("Missing denied annotation %s", exp)
		}
	}

	expectedMandatoryAnnotations := []string{"owner"}
	for _, exp := range expectedMandatoryAnnotations {
		if !settings.MandatoryAnnotations.Contains(exp) {
			t.Errorf("Missing mandatory annotation %s", exp)
		}
	}

	re, found := settings.ConstrainedAnnotations["cost-center"]
	if !found {
		t.Error("Didn't find the expected constrained annotation")
	}

	expectedRegexp := `cc-\d+`
	if re.String() != expectedRegexp {
		t.Errorf("Execpted regexp to be %v - got %v instead",
			expectedRegexp, re.String())
	}
}

func TestParseSettingsWithInvalidRegexp(t *testing.T) {
	settingsJSON := []byte(`
	{
		"denied_annotations": [ "foo", "bar" ],
		"mandatory_annotations": ["owner"],
		"constrained_annotations": {
			"cost-center": "cc-[a+"
		}
	}`)

	err := json.Unmarshal(settingsJSON, &Settings{})
	if err == nil {
		t.Errorf("Didn'g get expected error")
	}
}

func TestParseSettingsWithNullRegexp(t *testing.T) {
	err := json.Unmarshal([]byte(`{"constrained_annotations":{"cost-center":null}}`), &Settings{})
	if err == nil {
		t.Fatal("expected null regular expression to be rejected")
	}
}

func TestDetectValidSettings(t *testing.T) {
	request := `
	{
		"denied_annotations": [ "foo", "bar" ],
		"mandatory_annotations": ["owner"],
		"constrained_annotations": {
			"cost-center": "cc-\\d+"
		}
	}
	`
	rawRequest := []byte(request)
	responsePayload, err := validateSettings(rawRequest)
	if err != nil {
		t.Errorf("Unexpected error %+v", err)
	}

	var response kubewarden_protocol.SettingsValidationResponse
	if err := json.Unmarshal(responsePayload, &response); err != nil {
		t.Errorf("Unexpected error: %+v", err)
	}

	if !response.Valid {
		t.Errorf("Expected settings to be valid: %s", *response.Message)
	}
}

func TestDetectNotValidSettingsDueToBrokenRegexp(t *testing.T) {
	request := `
	{
		"denied_annotations": [ "foo", "bar" ],
		"mandatory_annotations": ["owner"],
		"constrained_annotations": {
			"cost-center": "cc-[a+"
		}
	}
	`
	rawRequest := []byte(request)
	responsePayload, err := validateSettings(rawRequest)
	if err != nil {
		t.Errorf("Unexpected error %+v", err)
	}

	var response kubewarden_protocol.SettingsValidationResponse
	if err := json.Unmarshal(responsePayload, &response); err != nil {
		t.Errorf("Unexpected error: %+v", err)
	}

	if response.Valid {
		t.Error("Expected settings to not be valid")
	}

	if *response.Message != "Provided settings are not valid: error parsing regexp: missing closing ]: `[a+`" {
		t.Errorf("Unexpected validation error message: %s", *response.Message)
	}
}

func TestDetectNotValidSettingsDueToConflictingDeniedAndConstrainedAnnotations(t *testing.T) {
	request := `
	{
		"denied_annotations": [ "foo", "bar", "cost-center" ],
		"mandatory_annotations": ["owner"],
		"constrained_annotations": {
			"cost-center": ".*"
		}
	}
	`
	rawRequest := []byte(request)
	responsePayload, err := validateSettings(rawRequest)
	if err != nil {
		t.Errorf("Unexpected error %+v", err)
	}

	var response kubewarden_protocol.SettingsValidationResponse
	if err := json.Unmarshal(responsePayload, &response); err != nil {
		t.Errorf("Unexpected error: %+v", err)
	}

	if response.Valid {
		t.Error("Expected settings to not be valid")
	}

	if *response.Message != "Provided settings are not valid: These annotations cannot be constrained and denied at the same time: cost-center" {
		t.Errorf("Unexpected validation error message: %s", *response.Message)
	}
}

func TestDetectNotValidSettingsDueToConflictingDeniedAndMandatoryAnnotations(t *testing.T) {
	request := `
	{
		"denied_annotations": [ "foo", "bar", "owner" ],
		"mandatory_annotations": ["owner"],
		"constrained_annotations": {
			"cost-center": ".*"
		}
	}
	`
	rawRequest := []byte(request)
	responsePayload, err := validateSettings(rawRequest)
	if err != nil {
		t.Errorf("Unexpected error %+v", err)
	}

	var response kubewarden_protocol.SettingsValidationResponse
	if err := json.Unmarshal(responsePayload, &response); err != nil {
		t.Errorf("Unexpected error: %+v", err)
	}

	if response.Valid {
		t.Error("Expected settings to not be valid")
	}

	if *response.Message != "Provided settings are not valid: These annotations cannot be mandatory and denied at the same time: owner" {
		t.Errorf("Unexpected validation error message: %s", *response.Message)
	}
}
