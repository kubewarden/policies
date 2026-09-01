package main

import (
	"encoding/json"
	"errors"
	"fmt"

	kubewarden "github.com/kubewarden/policy-sdk-go"
	kubewarden_protocol "github.com/kubewarden/policy-sdk-go/protocol"
)

type Settings struct {
	PropagatedLabels []string `json:"propagatedLabels"`
}

func NewSettingsFromValidationReq(validationReq *kubewarden_protocol.ValidationRequest) (Settings, error) {
	settings := Settings{}
	err := json.Unmarshal(validationReq.Settings, &settings)
	return settings, err
}

// Valid checks that the Settings are valid. No special checks have to be done
// besides ensuring at least one non-empty label has been provided.
func (s *Settings) Valid() (bool, error) {
	if len(s.PropagatedLabels) == 0 {
		return false, errors.New("some label must be provided")
	}
	for _, label := range s.PropagatedLabels {
		if len(label) == 0 {
			return false, errors.New("empty labels are not allowed")
		}
	}
	return true, nil
}

func validateSettings(payload []byte) ([]byte, error) {
	logger.Info("validating settings")

	settings := Settings{}
	err := json.Unmarshal(payload, &settings)
	if err != nil {
		return kubewarden.RejectSettings(kubewarden.Message(fmt.Sprintf("Provided settings are not valid: %v", err)))
	}

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
