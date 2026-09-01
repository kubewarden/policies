package main

import (
	"testing"
)

func TestParsingSettingsWithAllValuesProvidedFromValidationReq(t *testing.T) {
	request := `
	{
		"request": "doesn't matter here",
		"settings": {
			"allowedTypes": [
				"configMap",
				"downwardAPI",
				"emptyDir",
				"persistentVolumeClaim",
				"secret",
				"projected"
			]
		}
	}
	`
	rawRequest := []byte(request)

	settings, err := NewSettingsFromValidationReq(rawRequest)
	if err != nil {
		t.Errorf("Unexpected error %+v", err)
	}

	if !settings.AllowedTypes.Contains("configMap") ||
		!settings.AllowedTypes.Contains("downwardAPI") ||
		!settings.AllowedTypes.Contains("emptyDir") ||
		!settings.AllowedTypes.Contains("persistentVolumeClaim") ||
		!settings.AllowedTypes.Contains("secret") ||
		!settings.AllowedTypes.Contains("projected") {
		t.Errorf("Missing value")
	}
}

func TestParsingSettingsWithNoValueProvided(t *testing.T) {
	request := `
	{
		"request": "doesn't matter here",
		"settings": {
		}
	}
	`
	rawRequest := []byte(request)

	settings, err := NewSettingsFromValidationReq(rawRequest)
	if err != nil {
		t.Errorf("Unexpected error %+v", err)
	}

	if settings.AllowedTypes.Cardinality() != 0 {
		t.Errorf("Expected AllowedTypes to be empty")
	}
}

func TestSettingsWithInvalidEntries(t *testing.T) {
	request := `
	{
		"allowedTypes": [
			"configMap",
			"*"
		]
	}
	`
	rawRequest := []byte(request)

	settings, err := NewSettingsFromValidateSettingsPayload(rawRequest)
	if err != nil {
		t.Error("Expected no error, got one")
	}

	if settings.Valid() {
		t.Errorf("Expected Settings reported as not valid")
	}
}

func TestEmptySettingsAreValid(t *testing.T) {
	request := `
	{
		"request": "doesn't matter here",
		"settings": {
			"allowedTypes": []
		}
	}
	`
	rawRequest := []byte(request)

	settings, err := NewSettingsFromValidateSettingsPayload(rawRequest)
	if err != nil {
		t.Errorf("Unexpected error %+v", err)
	}

	if !settings.Valid() {
		t.Errorf("Settings are reported as not valid")
	}
}

func TestParsingSettingsWithIgnoreInitContainersVolumes(t *testing.T) {
	request := `
	{
		"request": "doesn't matter here",
		"settings": {
			"allowedTypes": ["configMap"],
			"ignoreInitContainersVolumes": true
		}
	}
	`
	rawRequest := []byte(request)

	settings, err := NewSettingsFromValidationReq(rawRequest)
	if err != nil {
		t.Errorf("Unexpected error %+v", err)
	}

	if !settings.IgnoreInitContainersVolumes {
		t.Errorf("Expected IgnoreInitContainersVolumes to be true")
	}

	if !settings.AllowedTypes.Contains("configMap") {
		t.Errorf("Missing allowedTypes value")
	}
}

func TestParsingSettingsWithIgnoreInitContainersVolumesDefault(t *testing.T) {
	request := `
	{
		"request": "doesn't matter here",
		"settings": {
			"allowedTypes": ["configMap"]
		}
	}
	`
	rawRequest := []byte(request)

	settings, err := NewSettingsFromValidationReq(rawRequest)
	if err != nil {
		t.Errorf("Unexpected error %+v", err)
	}

	if settings.IgnoreInitContainersVolumes {
		t.Errorf("Expected IgnoreInitContainersVolumes to be false by default")
	}
}
