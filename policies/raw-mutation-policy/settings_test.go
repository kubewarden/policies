package main

import (
	"encoding/json"
	"testing"

	mapset "github.com/deckarep/golang-set/v2"
)

func TestValidateSettingsAccept(t *testing.T) {
	settings := &Settings{
		ForbiddenResources: mapset.NewSet("banana"),
		DefaultResource:    "hay",
	}

	valid, err := settings.Valid()
	if !valid {
		t.Errorf("Settings are reported as not valid")
	}
	if err != nil {
		t.Errorf("Unexpected error %+v", err)
	}
}

func TestValidateSettingsWithNullSetDoesNotPanic(t *testing.T) {
	responseJSON, err := validateSettings([]byte(`{"forbiddenResources":null,"defaultResource":"hay"}`))
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	var response struct {
		Valid bool `json:"valid"`
	}
	if err := json.Unmarshal(responseJSON, &response); err != nil {
		t.Fatalf("cannot unmarshal response: %v", err)
	}
	if !response.Valid {
		t.Error("null set should use an initialized empty set")
	}
}

func TestValidateSettingsRejectDefaultResourceEmpty(t *testing.T) {
	settings := &Settings{
		ForbiddenResources: mapset.NewSet("banana"),
		DefaultResource:    "",
	}

	valid, err := settings.Valid()
	if valid {
		t.Errorf("Settings are reported as valid")
	}

	if err == nil {
		t.Errorf("Unexpected nil error")
	}
}

func TestValidateSettingsRejectDefaultResourceForbidden(t *testing.T) {
	settings := &Settings{
		ForbiddenResources: mapset.NewSet("banana"),
		DefaultResource:    "banana",
	}

	valid, err := settings.Valid()
	if valid {
		t.Errorf("Settings are reported as valid")
	}

	if err == nil {
		t.Errorf("Unexpected nil error")
	}
}
