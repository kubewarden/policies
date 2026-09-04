package main

import (
	"encoding/json"

	mapset "github.com/deckarep/golang-set/v2"
	"github.com/kubewarden/gjson"
	kubewarden "github.com/kubewarden/policy-sdk-go"
)

type Settings struct {
	AllowedTypes                mapset.Set[string] `json:"allowedTypes"`
	IgnoreInitContainersVolumes bool               `json:"ignoreInitContainersVolumes,omitempty"`
}

// NewSettingsFromValidationReq builds a new Settings instance starting from a validation
// request payload:
//
//	{
//	   "request": ...,
//	   "settings": {
//			"allowedTypes": [
//				"configMap",
//				"downwardAPI",
//				"emptyDir",
//				"persistentVolumeClaim",
//				"secret",
//				"projected"
//			]
//	   }
//	}
func NewSettingsFromValidationReq(payload []byte) (Settings, error) {
	settingsJSON := gjson.GetBytes(payload, "settings")
	settings := Settings{}

	err := json.Unmarshal([]byte(settingsJSON.Raw), &settings)
	if err != nil {
		return Settings{}, err
	}

	return settings, nil
}

// NewSettingsFromValidateSettingsPayload builds a new Settings instance starting from a Settings
// payload:
//
//	{
//		  "allowedTypes": [
//		  	"configMap",
//		  	"downwardAPI",
//		  	"emptyDir",
//		  	"persistentVolumeClaim",
//		  	"secret",
//		  	"projected"
//		  ]
//	}
func NewSettingsFromValidateSettingsPayload(payload []byte) (Settings, error) {
	settings := Settings{}

	err := json.Unmarshal(payload, &settings)
	if err != nil {
		return Settings{}, err
	}

	return settings, nil
}

func (s *Settings) Valid() bool {
	if s.AllowedTypes.Contains("*") && (s.AllowedTypes.Cardinality() != 1) {
		return false
	}
	return true
}

func (s *Settings) UnmarshalJSON(data []byte) error {
	// This is needed becaus golang-set v2.3.0 has a bug that prevents
	// the correct unmarshalling of ThreadUnsafeSet types.
	rawSettings := struct {
		AllowedTypes                []string `json:"allowedTypes"`
		IgnoreInitContainersVolumes bool     `json:"ignoreInitContainersVolumes,omitempty"`
	}{}

	err := json.Unmarshal(data, &rawSettings)
	if err != nil {
		return err
	}

	s.AllowedTypes = mapset.NewThreadUnsafeSet[string](rawSettings.AllowedTypes...)
	s.IgnoreInitContainersVolumes = rawSettings.IgnoreInitContainersVolumes

	return nil
}

func validateSettings(payload []byte) ([]byte, error) {
	logger.Info("validating settings")

	settings, err := NewSettingsFromValidateSettingsPayload(payload)
	if err != nil {
		return kubewarden.RejectSettings(kubewarden.Message(err.Error()))
	}

	if settings.Valid() {
		return kubewarden.AcceptSettings()
	}

	logger.Warn("rejecting settings")
	return kubewarden.RejectSettings(kubewarden.Message("Provided settings are not valid"))
}
