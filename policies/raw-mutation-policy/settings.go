package main

import (
	"encoding/json"
	"fmt"

	mapset "github.com/deckarep/golang-set/v2"
	kubewarden "github.com/kubewarden/policy-sdk-go"
)

// Settings is the structure that describes the policy settings.
type Settings struct {
	ForbiddenResources mapset.Set[string] `json:"forbiddenResources"`
	DefaultResource    string             `json:"defaultResource"`
}

func (s *Settings) ensureSets() {
	if s.ForbiddenResources == nil {
		s.ForbiddenResources = mapset.NewSet[string]()
	}
}

func (s *Settings) Valid() (bool, error) {
	if s.DefaultResource == "" {
		return false, fmt.Errorf("defaultResource cannot be empty")
	}

	if s.ForbiddenResources.Contains(s.DefaultResource) {
		return false, fmt.Errorf("defaultResource cannot be forbidden")
	}

	return true, nil
}

func validateSettings(payload []byte) ([]byte, error) {
	logger.Info("validating settings")

	settings := Settings{
		ForbiddenResources: mapset.NewSet[string](),
	}
	err := json.Unmarshal(payload, &settings)
	if err != nil {
		return kubewarden.RejectSettings(kubewarden.Message(fmt.Sprintf("Provided settings are not valid: %v", err)))
	}
	settings.ensureSets()

	valid, err := settings.Valid()
	if err != nil {
		return kubewarden.RejectSettings(kubewarden.Message(fmt.Sprintf("Provided settings are not valid: %v", err)))
	}
	if valid {
		return kubewarden.AcceptSettings()
	}

	logger.Warn("rejecting settings")
	return kubewarden.RejectSettings(kubewarden.Message("Provided settings are not valid"))
}
