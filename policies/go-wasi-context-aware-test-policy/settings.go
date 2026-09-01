package main

import (
	"encoding/json"
	"fmt"

	mapset "github.com/deckarep/golang-set/v2"
	kubewarden "github.com/kubewarden/policy-sdk-go"
	kubewardenProtocol "github.com/kubewarden/policy-sdk-go/protocol"
)

// Settings defines the settings of the policy.
type Settings struct {
	RequiredAnnotations  map[string]string  `json:"requiredAnnotations"`
	ForbiddenAnnotations mapset.Set[string] `json:"forbiddenAnnotations"`
}

func NewSettingsFromValidationReq(validationReq *kubewardenProtocol.ValidationRequest) (Settings, error) {
	settings := Settings{
		// this is required to make the unmarshal work
		ForbiddenAnnotations: mapset.NewSet[string](),
	}

	err := json.Unmarshal(validationReq.Settings, &settings)
	if err != nil {
		return Settings{}, fmt.Errorf("cannot unmarshal settings %w", err)
	}
	return settings, nil
}

func validateSettings(input []byte) ([]byte, error) {
	settings := &Settings{
		// this is required to make the unmarshal work
		ForbiddenAnnotations: mapset.NewSet[string](),
	}
	err := json.Unmarshal(input, &settings)
	if err != nil {
		return kubewarden.RejectSettings(kubewarden.Message(fmt.Sprintf("cannot unmarshal settings: %v", err)))
	}

	return validateCliSettings(settings)
}

func validateCliSettings(settings *Settings) ([]byte, error) {
	required := mapset.NewSet[string]()
	for key := range settings.RequiredAnnotations {
		required.Add(key)
	}

	forbiddenButRequired := settings.ForbiddenAnnotations.Intersect(required)

	if forbiddenButRequired.Cardinality() > 0 {
		return kubewarden.RejectSettings(kubewarden.Message(
			"The following annotations are forbidden and required at the same time: " + forbiddenButRequired.String()))
	}

	return kubewarden.AcceptSettings()
}
