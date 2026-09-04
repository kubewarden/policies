package main

import (
	"encoding/json"
	"fmt"
	"log"

	mapset "github.com/deckarep/golang-set/v2"
)

// Settings defines the settings of the policy.
type Settings struct {
	ValidUsers     mapset.Set[string] `json:"validUsers"`
	ValidActions   mapset.Set[string] `json:"validActions"`
	ValidResources mapset.Set[string] `json:"validResources"`
}

func validateSettings(input []byte) []byte {
	var response SettingsValidationResponse

	settings := &Settings{
		// this is required to make the unmarshal work
		ValidUsers:     mapset.NewSet[string](),
		ValidActions:   mapset.NewSet[string](),
		ValidResources: mapset.NewSet[string](),
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
	if settings.ValidUsers.Cardinality() == 0 {
		return RejectSettings(Message(
			"At least one valid user must be specified"))
	}

	if settings.ValidActions.Cardinality() == 0 {
		return RejectSettings(Message(
			"At least one valid action must be specified"))
	}

	if settings.ValidResources.Cardinality() == 0 {
		return RejectSettings(Message(
			"At least one valid resource must be specified"))
	}

	return AcceptSettings()
}
