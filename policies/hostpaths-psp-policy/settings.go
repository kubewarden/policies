package main

import (
	"errors"
	"fmt"

	"github.com/kubewarden/gjson"
	kubewarden "github.com/kubewarden/policy-sdk-go"
)

type HostPath struct {
	PathPrefix string `json:"pathPrefix"`
	ReadOnly   bool   `json:"readOnly"`
}

type Settings struct {
	AllowedHostPaths []HostPath `json:"allowedHostPaths"`
}

// NewSettingsFromValidationReq builds a new Settings instance starting from a validation
// request payload:
//
//	{
//	   "request": ...,
//	   "settings": {
//	      "allowedHostPaths": [
//	      	{
//	      	  "pathPrefix": "foo",
//	      	  "readOnly": true,
//	         }
//	      ]
//	   }
//	}
func NewSettingsFromValidationReq(payload []byte) (Settings, error) {
	return newSettings(
		payload,
		"settings.allowedHostPaths")
}

// NewSettingsFromValidateSettingsPayload builds a new Settings instance starting from a Settings
// payload:
//
//	{
//	  "allowedHostPaths": [
//	  	{
//	  	  "pathPrefix": "foo",
//	  	  "readOnly": true,
//	     }
//	  ]
//	}
func NewSettingsFromValidateSettingsPayload(payload []byte) (Settings, error) {
	return newSettings(
		payload,
		"allowedHostPaths")
}

func newSettings(payload []byte, paths ...string) (Settings, error) {
	if len(paths) != 1 {
		return Settings{}, fmt.Errorf("wrong number of json paths")
	}

	data := gjson.GetManyBytes(payload, paths...)

	var err error
	allowedHostPaths := make([]HostPath, 0)

	if data[0].String() == "" {
		// empty settings
		return Settings{
			AllowedHostPaths: allowedHostPaths,
		}, nil
	}

	data[0].ForEach(func(_, entry gjson.Result) bool {
		dataHostPath := gjson.GetManyBytes([]byte(entry.String()),
			"pathPrefix",
			"readOnly",
		)
		if !dataHostPath[0].Exists() {
			errMsg := "pathPrefix key is missing"
			if err == nil {
				err = errors.New(errMsg)
			} else {
				err = fmt.Errorf("%w; %s", err, errMsg)
			}
			return true // continue iterating
		}
		if !dataHostPath[1].Exists() {
			errMsg := fmt.Sprintf("readOnly key for pathPrefix '%s' is missing",
				dataHostPath[0].String())
			if err == nil {
				err = errors.New(errMsg)
			} else {
				err = fmt.Errorf("%w; %s", err, errMsg)
			}
			return true // continue iterating
		}

		hostPath := HostPath{
			PathPrefix: dataHostPath[0].String(),
			ReadOnly:   dataHostPath[1].Bool(),
		}
		allowedHostPaths = append(allowedHostPaths, hostPath)
		return true // continue iterating
	})

	return Settings{
		AllowedHostPaths: allowedHostPaths,
	}, err
}

func (s *Settings) Valid() bool {
	// each entry of allowedHostPaths needs to have 1 pathPrefix and 1 readOnly,
	// which is checked on marshalling
	return true
}

func validateSettings(payload []byte) ([]byte, error) {
	logger.Info("validating settings")

	settings, err := NewSettingsFromValidateSettingsPayload(payload)
	if err != nil {
		return kubewarden.RejectSettings(kubewarden.Message(err.Error()))
	}

	if settings.Valid() {
		logger.Info("accepting settings")
		return kubewarden.AcceptSettings()
	}

	logger.Warn("rejecting settings")
	return kubewarden.RejectSettings(kubewarden.Message("Provided settings are not valid"))
}
