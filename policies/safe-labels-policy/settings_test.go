package main

import (
	"encoding/json"
	"testing"

	kubewarden_protocol "github.com/kubewarden/policy-sdk-go/protocol"
)

func TestParseValidSettings(t *testing.T) {
	request := `
	{
		"request": "doesn't matter here",
		"settings": {
			"denied_labels": [ "foo", "bar" ],
			"mandatory_labels": ["owner"],
			"constrained_labels": {
				"cost-center": "cc-\\d+"
			}
		}
	}
	`
	rawRequest := []byte(request)

	settings, err := NewSettingsFromValidationReq(rawRequest)
	if err != nil {
		t.Errorf("Unexpected error %+v", err)
	}

	expectedDeniedLabels := []string{"foo", "bar"}
	for _, exp := range expectedDeniedLabels {
		if !settings.DeniedLabels.Contains(exp) {
			t.Errorf("Missing denied label %s", exp)
		}
	}

	expectedMandatoryLabels := []string{"owner"}
	for _, exp := range expectedMandatoryLabels {
		if !settings.MandatoryLabels.Contains(exp) {
			t.Errorf("Missing mandatory label %s", exp)
		}
	}

	re, found := settings.ConstrainedLabels["cost-center"]
	if !found {
		t.Error("Didn't find the expected constrained label")
	}

	expectedRegexp := `cc-\d+`
	if re.String() != expectedRegexp {
		t.Errorf("Execpted regexp to be %v - got %v instead",
			expectedRegexp, re.String())
	}
}

func TestParseSettingsWithInvalidRegexp(t *testing.T) {
	request := `
	{
		"request": "doesn't matter here",
		"settings": {
			"denied_labels": [ "foo", "bar" ],
			"mandatory_labels": ["owner"],
			"constrained_labels": {
				"cost-center": "cc-[a+"
			}
		}
	}
	`
	rawRequest := []byte(request)

	_, err := NewSettingsFromValidationReq(rawRequest)
	if err == nil {
		t.Errorf("Didn'g get expected error")
	}
}

func TestParseSettingsWithNullRegexp(t *testing.T) {
	_, err := NewSettingsFromValidateSettingsPayload([]byte(`{"constrained_labels":{"cost-center":null}}`))
	if err == nil {
		t.Fatal("expected null regular expression to be rejected")
	}
}

func TestDetectValidSettings(t *testing.T) {
	request := `
	{
		"denied_labels": [ "foo", "bar" ],
		"mandatory_labels": ["owner"],
		"constrained_labels": {
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
		"denied_labels": [ "foo", "bar" ],
		"mandatory_labels": ["owner"],
		"constrained_labels": {
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

func TestDetectNotValidSettingsDueToConflictingDeniedAndConstrainedLabels(t *testing.T) {
	request := `
	{
		"denied_labels": [ "foo", "bar", "cost-center" ],
		"mandatory_labels": ["owner"],
		"constrained_labels": {
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

	expectedErrorMsg := "Provided settings are not valid: These labels cannot be constrained and denied at the same time: cost-center"
	if *response.Message != expectedErrorMsg {
		t.Errorf("Unexpected validation error message: %s", *response.Message)
	}
}

func TestDetectNotValidSettingsDueToConflictingDeniedAndMandatoryLabels(t *testing.T) {
	request := `
	{
		"denied_labels": [ "foo", "bar", "owner"],
		"mandatory_labels": ["owner"],
		"constrained_labels": {
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

	expectedErrorMsg := "Provided settings are not valid: These labels cannot be mandatory and denied at the same time: owner"
	if *response.Message != expectedErrorMsg {
		t.Errorf("Unexpected validation error message: %s", *response.Message)
	}
}
