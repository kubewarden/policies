package main

import (
	"encoding/json"
	"testing"
)

func TestValidateSettingsNullDoesNotPanic(t *testing.T) {
	responseJSON := validateSettings([]byte("null"))
	var response SettingsValidationResponse
	if err := json.Unmarshal(responseJSON, &response); err != nil {
		t.Fatalf("cannot unmarshal response: %v", err)
	}

	if response.Valid {
		t.Error("null settings should be invalid")
	}
}

func TestValidateSettingsRejectsEmptyPolicy(t *testing.T) {
	for _, payload := range []string{`{}`, `{"policy":null}`, `{"policy":""}`} {
		responseJSON := validateSettings([]byte(payload))
		var response SettingsValidationResponse
		if err := json.Unmarshal(responseJSON, &response); err != nil {
			t.Fatalf("cannot unmarshal response: %v", err)
		}
		if response.Valid {
			t.Errorf("settings %s should be invalid", payload)
		}
	}
}
