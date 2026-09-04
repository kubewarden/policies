package main

import (
	"encoding/json"
	"testing"

	mapset "github.com/deckarep/golang-set/v2"
)

const defaultResourceHay = "hay"

func TestValidateSettingsAccept(t *testing.T) {
	settings := &Settings{
		ForbiddenResources: mapset.NewSet("banana"),
		DefaultResource:    defaultResourceHay,
	}
	settingsJSON, err := json.Marshal(&settings)
	if err != nil {
		t.Errorf("cannot marshal settings: %v", err)
	}

	responseJSON := validateSettings(settingsJSON)
	var response SettingsValidationResponse
	err = json.Unmarshal(responseJSON, &response)
	if err != nil {
		t.Errorf("cannot unmarshal response: %v", err)
	}

	if !response.Valid {
		t.Errorf("response should be valid")
	}
}

func TestValidateSettingsRejectDefaultResourceEmpty(t *testing.T) {
	settings := &Settings{
		ForbiddenResources: mapset.NewSet("banana"),
		DefaultResource:    "",
	}
	settingsJSON, err := json.Marshal(&settings)
	if err != nil {
		t.Errorf("cannot marshal settings: %v", err)
	}

	responseJSON := validateSettings(settingsJSON)
	var response SettingsValidationResponse
	err = json.Unmarshal(responseJSON, &response)
	if err != nil {
		t.Errorf("cannot unmarshal response: %v", err)
	}

	if response.Valid {
		t.Errorf("response should be invalid")
	}
}

func TestValidateSettingsRejectDefaultResourceForbidden(t *testing.T) {
	settings := &Settings{
		ForbiddenResources: mapset.NewSet("banana"),
		DefaultResource:    "banana",
	}
	settingsJSON, err := json.Marshal(&settings)
	if err != nil {
		t.Errorf("cannot marshal settings: %v", err)
	}

	responseJSON := validateSettings(settingsJSON)
	var response SettingsValidationResponse
	err = json.Unmarshal(responseJSON, &response)
	if err != nil {
		t.Errorf("cannot unmarshal response: %v", err)
	}

	if response.Valid {
		t.Errorf("response should be invalid")
	}
}
