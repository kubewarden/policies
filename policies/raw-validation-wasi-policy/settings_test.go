package main

import (
	"encoding/json"
	"testing"

	mapset "github.com/deckarep/golang-set/v2"
)

func TestValidateSettingsAccept(t *testing.T) {
	settings := &Settings{
		ValidUsers:     mapset.NewSet[string]("tonio", "wanda"),
		ValidActions:   mapset.NewSet[string]("eats", "likes"),
		ValidResources: mapset.NewSet[string]("hay", "carrot", "banana"),
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

func TestValidateSettingsReject(t *testing.T) {
	settings := &Settings{
		ValidUsers:     mapset.NewSet[string](),
		ValidActions:   mapset.NewSet[string](),
		ValidResources: mapset.NewSet[string](),
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

	if *response.Message != "At least one valid user must be specified" {
		t.Errorf("wrong message: %s", *response.Message)
	}
}

func TestValidateSettingsNullDoesNotPanic(t *testing.T) {
	responseJSON := validateSettings([]byte("null"))

	var response SettingsValidationResponse

	err := json.Unmarshal(responseJSON, &response)
	if err != nil {
		t.Fatalf("cannot unmarshal response: %v", err)
	}

	if response.Valid {
		t.Error("null settings should be invalid")
	}
}

func TestValidateSettingsWithNullSetDoesNotPanic(t *testing.T) {
	responseJSON := validateSettings([]byte(`{"validUsers":null,"validActions":["get"],"validResources":["pods"]}`))

	var response SettingsValidationResponse

	err := json.Unmarshal(responseJSON, &response)
	if err != nil {
		t.Fatalf("cannot unmarshal response: %v", err)
	}

	if response.Valid {
		t.Error("empty validUsers should be invalid")
	}
}
