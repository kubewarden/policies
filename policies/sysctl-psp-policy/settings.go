package main

import (
	"encoding/json"

	mapset "github.com/deckarep/golang-set/v2"
	"github.com/kubewarden/gjson"
	kubewarden "github.com/kubewarden/policy-sdk-go"

	"fmt"
	"strings"
)

type Settings struct {
	AllowedUnsafeSysctls mapset.Set[string] `json:"allowedUnsafeSysctls"`
	ForbiddenSysctls     mapset.Set[string] `json:"forbiddenSysctls"`
}

// NewSettingsFromValidationReq builds a new Settings instance starting from a validation
// request payload:
//
//	{
//	   "request": ...,
//	   "settings": {
//	      "allowedUnsafeSysctls": [...],
//	      "forbiddenSysctls": [...]
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
//	   "allowedUnsafeSysctls": [...],
//	   "forbiddenSysctls": [...]
//	}
func NewSettingsFromValidateSettingsPayload(payload []byte) (Settings, error) {
	settings := Settings{}
	err := json.Unmarshal(payload, &settings)
	if err != nil {
		return Settings{}, err
	}

	return settings, nil
}

func (s *Settings) UnmarshalJSON(data []byte) error {
	// This is needed becaus golang-set v2.3.0 has a bug that prevents
	// the correct unmarshalling of ThreadUnsafeSet types.
	rawSettings := struct {
		AllowedUnsafeSysctls []string `json:"allowedUnsafeSysctls"`
		ForbiddenSysctls     []string `json:"forbiddenSysctls"`
	}{}

	err := json.Unmarshal(data, &rawSettings)
	if err != nil {
		return err
	}

	s.AllowedUnsafeSysctls = mapset.NewThreadUnsafeSet(rawSettings.AllowedUnsafeSysctls...)
	s.ForbiddenSysctls = mapset.NewThreadUnsafeSet(rawSettings.ForbiddenSysctls...)

	return nil
}

func (s *Settings) Valid() (bool, error) {
	for _, elem := range s.AllowedUnsafeSysctls.ToSlice() {
		if strings.Contains(elem, "*") {
			return false,
				fmt.Errorf("allowedUnsafeSysctls doesn't accept patterns with `*`")
		}
	}

	for _, elem := range s.ForbiddenSysctls.ToSlice() {
		if strings.Contains(elem, "*") &&
			!strings.HasSuffix(elem, "*") {
			return false,
				fmt.Errorf("forbiddenSysctls only accepts patterns with `*` as suffix")
		}
	}

	allowedAndForbidden := s.AllowedUnsafeSysctls.Intersect(s.ForbiddenSysctls)
	if allowedAndForbidden.Cardinality() != 0 {
		return false,
			fmt.Errorf("these sysctls cannot be allowed and forbidden at the same time: %s",
				strings.Join(allowedAndForbidden.ToSlice(), ","),
			)
	}

	return true, nil
}

func validateSettings(payload []byte) ([]byte, error) {
	logger.Info("validating settings")

	settings, err := NewSettingsFromValidateSettingsPayload(payload)
	if err != nil {
		return kubewarden.RejectSettings(
			kubewarden.Message(fmt.Sprintf("provided settings are not valid: %v", err)))
	}

	valid, err := settings.Valid()
	if valid {
		return kubewarden.AcceptSettings()
	}

	logger.Warn("rejecting settings")
	return kubewarden.RejectSettings(
		kubewarden.Message(fmt.Sprintf("provided settings are not valid: %v", err)))
}
