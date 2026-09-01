package main

import (
	"encoding/json"
	"fmt"
	"log"

	mapset "github.com/deckarep/golang-set/v2"
)

// Settings defines the settings of the policy.
type Settings struct {
	ForbiddenResources mapset.Set[string] `json:"forbiddenResources"`
	DefaultResource    string             `json:"defaultResource"`
}

func validateSettings(input []byte) []byte {
	var response SettingsValidationResponse

	settings := &Settings{
		// this is required to make the unmarshal work
		ForbiddenResources: mapset.NewSet[string](),
	}
	err := json.Unmarshal(input, &settings)
	if err != nil {
		response = RejectSettings(Message(fmt.Sprintf("cannot unmarshal settings: %v", err)))
	} else {
		response = validateCliSettings(settings)
	}

	responseBytes, err := json.Marshal(&response)
	if err != nil {
		log.Fatalf("cannot marshal validation response: %v", err)
	}
	return responseBytes
}

func validateCliSettings(settings *Settings) SettingsValidationResponse {
	if settings.DefaultResource == "" {
		return RejectSettings(Message("default resource cannot be empty"))
	}

	if settings.ForbiddenResources.Contains(settings.DefaultResource) {
		return RejectSettings(Message("default resource cannot be forbidden"))
	}

	return AcceptSettings()
}
