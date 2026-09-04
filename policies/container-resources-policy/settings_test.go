package main

import (
	"encoding/json"
	"errors"
	"testing"

	"github.com/kubewarden/container-resources-policy/resource"
	kubewarden_protocol "github.com/kubewarden/policy-sdk-go/protocol"
	"github.com/stretchr/testify/require"
)

func checkSettingsValues(
	t *testing.T,
	settings *ResourceConfiguration,
	expectedMaxLimit, expectedDefaultRequest, expectedDefaultLimit string,
	expectedIgnoreValues bool,
) {
	actualMaxLimit := resource.MustParse(expectedMaxLimit)
	if !settings.MaxLimit.Equal(actualMaxLimit) {
		t.Errorf("invalid max limit quantity parsed. Expected %+v, got %+v", actualMaxLimit, settings.MaxLimit)
	}
	actualDefaultRequest := resource.MustParse(expectedDefaultRequest)
	if !settings.DefaultRequest.Equal(actualDefaultRequest) {
		t.Errorf(
			"invalid default request quantity parsed. Expected %+v, got %+v",
			actualDefaultRequest,
			settings.DefaultRequest,
		)
	}
	actualDefaultLimit := resource.MustParse(expectedDefaultLimit)
	if !settings.DefaultLimit.Equal(actualDefaultLimit) {
		t.Errorf(
			"invalid default limit quantity parsed. Expected %+v, got %+v",
			actualDefaultLimit,
			settings.DefaultLimit,
		)
	}
	if settings.IgnoreValues != expectedIgnoreValues {
		t.Errorf("invalid ignoreValues value. Expected %t, got %t", expectedIgnoreValues, settings.IgnoreValues)
	}
}

func TestParsingResourceConfiguration(t *testing.T) {
	tests := []struct {
		name        string
		rawSettings []byte
		err         error
	}{
		{
			name:        "no suffix",
			rawSettings: []byte(`{"maxLimit": "3", "minRequest": "1", "defaultLimit": "2", "defaultRequest": "1"}`),
		},
		{
			name: "valid ignoreValues with valid resource configuration",
			rawSettings: []byte(
				`{"maxLimit": "3", "minRequest": "1", "defaultLimit": "2", "defaultRequest": "1", "ignoreValues": true}`,
			),
		},
		{
			name: "valid ignoreValues",
			rawSettings: []byte(
				`{"maxLimit": "3", "minRequest": "1", "defaultLimit": "2", "defaultRequest": "1", "ignoreValues": false}`,
			),
		},
		{
			name:        "valid ignoreValues",
			rawSettings: []byte(`{"ignoreValues": true}`),
		},
		{
			name:        "invalid ignoreValues",
			rawSettings: []byte(`{"ignoreValues": false}`),
			err:         AllValuesAreZeroError{},
		},
		{
			name:        "invalid limit suffix",
			rawSettings: []byte(`{"maxLimit": "1x", "minRequest": "1x", "defaultLimit": "1m", "defaultRequest": "1m"}`),
			err: errors.New(
				"quantities must match the regular expression '^([+-]?[0-9.]+)([eEinumkKMGTP]*[-+]?[0-9]*)$'",
			),
		},
		{
			name:        "invalid request suffix",
			rawSettings: []byte(`{"maxLimit": "3m", "minRequest": "1m", "defaultLimit": "2m", "defaultRequest": "1x"}`),
			err: errors.New(
				"quantities must match the regular expression '^([+-]?[0-9.]+)([eEinumkKMGTP]*[-+]?[0-9]*)$'",
			),
		},
		{
			name:        "valid resource configuration",
			rawSettings: []byte(`{"maxLimit": "4G", "defaultLimit": "2G", "defaultRequest": "1G"}`),
		},
		{
			name:        "valid resource configuration with all fields",
			rawSettings: []byte(`{"minRequest": "1G", "maxLimit": "4G", "defaultLimit": "2G", "defaultRequest": "1G"}`),
		},
		{
			name:        "valid: minLimit configuration",
			rawSettings: []byte(`{"minLimit": "1m", "defaultLimit": "2m", "defaultRequest": "1m"}`),
		},
		{
			name:        "valid: maxRequest configuration",
			rawSettings: []byte(`{"maxRequest": "2m", "defaultLimit": "3m", "defaultRequest": "1m"}`),
		},
		{
			name:        "valid: minLimit with maxLimit consistency",
			rawSettings: []byte(`{"minLimit": "1m", "maxLimit": "4m", "defaultLimit": "2m", "defaultRequest": "1m"}`),
		},
		{
			name: "valid: maxRequest with minRequest consistency",
			rawSettings: []byte(
				`{"minRequest": "1m", "maxRequest": "2m", "defaultLimit": "3m", "defaultRequest": "2m"}`,
			),
		},
		{
			name:        "setting only maxLimit",
			rawSettings: []byte(`{"maxLimit": "3m"}`),
		},
		{
			name:        "setting only minLimit",
			rawSettings: []byte(`{"minLimit": "3m"}`),
		},
		{
			name:        "setting only maxRequest",
			rawSettings: []byte(`{"maxRequest": "3m"}`),
		},
		{
			name:        "setting only minRequest",
			rawSettings: []byte(`{"minRequest": "3m"}`),
		},
		{
			name:        "setting only defaultLimit",
			rawSettings: []byte(`{"defaultLimit": "3m"}`),
		},
		{
			name:        "setting only defaultRequest",
			rawSettings: []byte(`{"defaultRequest": "3m"}`),
		},
		{
			name:        "invalid: defaultLimit > maxLimit",
			rawSettings: []byte(`{"maxLimit": "2m", "defaultRequest": "3m", "defaultLimit": "4m"}`),
			err:         errors.New("default limit: 4m cannot be greater than max limit: 2m"),
		},
		{
			name:        "invalid: defaultLimit < minRequest",
			rawSettings: []byte(`{"minRequest": "4m", "defaultRequest": "3m", "defaultLimit": "4m"}`),
			err:         errors.New("min request: 4m cannot be greater than default request: 3m"),
		},
		{
			name:        "invalid: minLimit > defaultLimit",
			rawSettings: []byte(`{"minLimit": "3m", "defaultLimit": "2m", "defaultRequest": "1m"}`),
			err:         errors.New("min limit: 3m cannot be greater than default limit: 2m"),
		},
		{
			name:        "invalid: minLimit > defaultRequest",
			rawSettings: []byte(`{"minLimit": "2m", "defaultLimit": "3m", "defaultRequest": "1m"}`),
		},
		{
			name:        "invalid: maxRequest < defaultLimit",
			rawSettings: []byte(`{"maxRequest": "1m", "defaultLimit": "2m", "defaultRequest": "3m"}`),
			err:         errors.New("default request: 3m cannot be greater than default limit: 2m"),
		},
		{
			name:        "invalid: maxRequest < defaultRequest",
			rawSettings: []byte(`{"maxRequest": "1m", "defaultLimit": "3m", "defaultRequest": "2m"}`),
			err:         errors.New("default request: 2m cannot be greater than max request: 1m"),
		},
		{
			name:        "valid: complete constraint chain: minRequest <= maxRequest <= minLimit <= maxLimit",
			rawSettings: []byte(`{"minRequest": "1m", "maxRequest": "2m", "minLimit": "3m", "maxLimit": "4m"}`),
		},
		{
			name: "valid: equal values in constraint chain",
			rawSettings: []byte(
				`{"minRequest": "2m", "maxRequest": "2m", "minLimit": "2m", "maxLimit": "2m", "defaultLimit": "2m", "defaultRequest": "2m"}`,
			),
		},
		{
			name: "valid: maxRequest equals minLimit",
			rawSettings: []byte(
				`{"minRequest": "1m", "maxRequest": "3m", "minLimit": "3m", "maxLimit": "4m", "defaultLimit": "3m", "defaultRequest": "3m"}`,
			),
		},
		{
			name:        "invalid: minRequest > minLimit",
			rawSettings: []byte(`{"minRequest": "4m", "minLimit": "3m", "maxLimit": "5m"}`),
			err:         errors.New("min request: 4m cannot be greater than min limit: 3m"),
		},
		{
			name:        "invalid: maxRequest > minLimit",
			rawSettings: []byte(`{"minRequest": "1m", "maxRequest": "4m", "minLimit": "3m", "maxLimit": "5m"}`),
			err:         errors.New("max request: 4m cannot be greater than min limit: 3m"),
		},
		{
			name:        "invalid: minLimit > maxLimit",
			rawSettings: []byte(`{"minRequest": "1m", "maxRequest": "2m", "minLimit": "5m", "maxLimit": "4m"}`),
			err:         errors.New("min limit: 5m cannot be greater than max limit: 4m"),
		},
		{
			name:        "invalid: minRequest > minLimit",
			rawSettings: []byte(`{"minRequest": "4m", "minLimit": "3m", "maxLimit": "6m"}`),
			err:         errors.New("min request: 4m cannot be greater than min limit: 3m"),
		},
		{
			name:        "invalid: maxRequest > maxLimit",
			rawSettings: []byte(`{"maxRequest": "5m", "maxLimit": "4m"}`),
			err:         errors.New("max request: 5m cannot be greater than max limit: 4m"),
		},
		{
			name:        "valid: minRequest and maxRequest only",
			rawSettings: []byte(`{"minRequest": "1m", "maxRequest": "2m"}`),
		},
		{
			name:        "valid: minLimit and maxLimit only",
			rawSettings: []byte(`{"minLimit": "3m", "maxLimit": "4m"}`),
		},
		{
			name:        "valid: maxRequest and minLimit together",
			rawSettings: []byte(`{"maxRequest": "2m", "minLimit": "3m"}`),
		},
	}
	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			settings := &ResourceConfiguration{}
			if err := json.Unmarshal(test.rawSettings, settings); err != nil {
				require.Error(t, err)
				require.Contains(t, err.Error(), test.err.Error())
				return
			}
			require.Equal(t, test.err, settings.valid())
		})
	}
}

func TestParsingSettings(t *testing.T) {
	tests := []struct {
		name         string
		rawSettings  []byte
		errorMessage string
		err          error
	}{
		{
			name:        "invalid settings",
			rawSettings: []byte(`{}`),
			err:         errors.New("no settings provided. At least one resource limit or request must be verified"),
		},
		{
			name: "valid settings",
			rawSettings: []byte(
				`{"cpu": {"maxLimit": "1m", "defaultRequest": "1m", "defaultLimit": "1m"}, "memory":{ "defaultLimit": "200M", "defaultRequest": "100M", "maxLimit": "500M"}, "ignoreImages": ["image:latest"]}`,
			),
		},
		{
			name:        "valid settings with cpu field only",
			rawSettings: []byte(`{"cpu": {"maxLimit": "1m", "defaultRequest": "1m", "defaultLimit": "1m"}}`),
		},
		{
			name:        "valid settings with memory fields only",
			rawSettings: []byte(`{"memory":{ "defaultLimit": "200M", "defaultRequest": "100M", "maxLimit": "500M"}}`),
		},
		{
			name: "no suffix",
			rawSettings: []byte(
				`{"cpu": {"maxLimit": "3", "defaultLimit": "2", "defaultRequest": "1"}, "memory": {"maxLimit": "3", "defaultLimit": "2", "defaultRequest": "1"}, "ignoreImages": []}`,
			),
		},
		{
			name: "invalid cpu settings",
			rawSettings: []byte(
				`{"cpu": {"maxLimit": "2m", "defaultRequest": "3m", "defaultLimit": "4m"}, "memory":{ "defaultLimit": "2G", "defaultRequest": "1G", "maxLimit": "3G"}, "ignoreImages": ["image:latest"]}`,
			),
			err: errors.New("default limit: 4m cannot be greater than max limit: 2m"),
		},
		{
			name: "invalid memory settings",
			rawSettings: []byte(
				`{"cpu": {"maxLimit": "2m", "defaultRequest": "1m", "defaultLimit": "1m"}, "memory":{ "defaultLimit": "2G", "defaultRequest": "3G", "maxLimit": "1G"}, "ignoreImages": ["image:latest"]}`,
			),
			err: errors.New("default limit: 2G cannot be greater than max limit: 1G"),
		},
		{
			name: "invalid cpu request",
			rawSettings: []byte(
				`{"cpu": {"minRequest": "2m", "maxLimit":	"4m", "defaultRequest": "1m", "defaultLimit": "3m"}}`,
			),
			err: errors.New("min request: 2m cannot be greater than default request: 1m"),
		},
		{
			name: "valid settings with empty memory settings",
			rawSettings: []byte(
				`{"cpu": {"maxLimit": "1m", "defaultRequest": "1m", "defaultLimit": "1m"}, "memory":{"ignoreValues": false}, "ignoreImages": ["image:latest"]}`,
			),
		},
		{
			name: "valid settings with empty cpu settings",
			rawSettings: []byte(
				`{"cpu": {"ignoreValues": false}, "memory":{ "defaultLimit": "200M", "defaultRequest": "100M", "maxLimit": "500M", "ignoreValues": false}, "ignoreImages": ["image:latest"]}`,
			),
		},
		{
			name: "invalid settings with empty cpu and memory settings",
			rawSettings: []byte(
				`{"cpu": {"ignoreValues": false}, "memory":{"ignoreValues": false}, "ignoreImages": ["image:latest"]}`,
			),
			err: errors.New(
				"invalid cpu settings\nall the quantities must be defined\ninvalid memory settings\nall the quantities must be defined",
			),
		},
		{
			name:        "invalid cpu minLimit",
			rawSettings: []byte(`{"cpu": {"minLimit": "3m", "defaultLimit": "2m", "defaultRequest": "1m"}}`),
			err:         errors.New("min limit: 3m cannot be greater than default limit: 2m"),
		},
		{
			name:        "invalid cpu maxRequest",
			rawSettings: []byte(`{"cpu": {"maxRequest": "1m", "defaultLimit": "2m", "defaultRequest": "3m"}}`),
			err:         errors.New("default request: 3m cannot be greater than default limit: 2m"),
		},
		{
			name:        "invalid memory minLimit",
			rawSettings: []byte(`{"memory": {"minLimit": "3G", "defaultLimit": "2G", "defaultRequest": "1G"}}`),
			err:         errors.New("min limit: 3G cannot be greater than default limit: 2G"),
		},
		{
			name:        "invalid memory maxRequest",
			rawSettings: []byte(`{"memory": {"maxRequest": "1G", "defaultLimit": "2G", "defaultRequest": "3G"}}`),
			err:         errors.New("default request: 3G cannot be greater than default limit: 2G"),
		},
		{
			name: "valid settings with minLimit and maxRequest",
			rawSettings: []byte(
				`{"cpu": {"minLimit": "3m", "maxLimit": "4m", "maxRequest": "2m", "defaultLimit": "3m", "defaultRequest": "2m"}, "memory": {"minLimit": "3G", "maxLimit": "4G", "maxRequest": "2G", "defaultLimit": "3G", "defaultRequest": "2G"}}`,
			),
		},
		{
			name: "invalid cpu maxRequest greater than maxLimit",
			rawSettings: []byte(
				`{"cpu": {"maxRequest": "5m", "maxLimit": "4m", "defaultLimit": "3m", "defaultRequest": "2m"}}`,
			),
			err: errors.New("max request: 5m cannot be greater than max limit: 4m"),
		},
		{
			name: "invalid cpu minRequest greater than minLimit",
			rawSettings: []byte(
				`{"cpu": {"minLimit": "2m", "minRequest": "3m", "defaultLimit": "4m", "defaultRequest": "1m"}}`,
			),
			err: errors.New("min request: 3m cannot be greater than min limit: 2m"),
		},
	}
	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			settings := &Settings{}
			if err := json.Unmarshal(test.rawSettings, settings); err != nil {
				require.Error(t, test.err)
				require.Contains(t, err.Error(), test.err.Error())
				return
			}
			validationErr := settings.Valid()
			if test.err == nil {
				require.NoError(t, validationErr)
			} else {
				require.Error(t, validationErr)
				require.Contains(t, validationErr.Error(), test.err.Error())
			}
		})
	}
}

func TestNewSettingsFromValidationReq(t *testing.T) {
	validationReq := &kubewarden_protocol.ValidationRequest{
		Settings: []byte(
			`{"cpu": {"maxLimit": "3m","defaultRequest": "1m", "defaultLimit": "2m"}, "memory":{"maxLimit": "3G","defaultRequest": "1G", "defaultLimit": "2G"}}`,
		),
	}
	settings, err := NewSettingsFromValidationReq(validationReq)
	if err != nil {
		t.Fatalf("Unexpected error %+v", err)
	}
	checkSettingsValues(t, settings.CPU, "3m", "1m", "2m", false)
	checkSettingsValues(t, settings.Memory, "3G", "1G", "2G", false)
}

func TestNewSettingsPartialFieldsOnlyFromValidationReq(t *testing.T) {
	t.Run("only memory fields", func(t *testing.T) {
		validationReq := &kubewarden_protocol.ValidationRequest{
			Settings: []byte(`{"memory":{"maxLimit": "3G","defaultRequest": "1G", "defaultLimit": "2G"}}`),
		}
		settings, err := NewSettingsFromValidationReq(validationReq)
		if err != nil {
			t.Fatalf("Unexpected error %+v", err)
		}

		if settings.CPU != nil {
			t.Fatal("cpu settings should be null")
		}
		checkSettingsValues(t, settings.Memory, "3G", "1G", "2G", false)
	})
	t.Run("only cpu fields", func(t *testing.T) {
		validationReq := &kubewarden_protocol.ValidationRequest{
			Settings: []byte(`{"cpu":{"maxLimit": "1","defaultRequest": "1", "defaultLimit": "1"}}`),
		}
		settings, err := NewSettingsFromValidationReq(validationReq)
		if err != nil {
			t.Fatalf("Unexpected error %+v", err)
		}

		if settings.Memory != nil {
			t.Fatal("memory settings should be null")
		}

		checkSettingsValues(t, settings.CPU, "1", "1", "1", false)
	})
	t.Run("only memory fields with ignoreValues", func(t *testing.T) {
		validationReq := &kubewarden_protocol.ValidationRequest{
			Settings: []byte(
				`{"memory":{"maxLimit": "3G","defaultRequest": "1G", "defaultLimit": "2G", "ignoreValues": true}}`,
			),
		}
		settings, err := NewSettingsFromValidationReq(validationReq)
		if err != nil {
			t.Fatalf("Unexpected error %+v", err)
		}
		if settings.CPU != nil {
			t.Fatal("cpu settings should be null")
		}
		checkSettingsValues(t, settings.Memory, "3G", "1G", "2G", true)
	})
	t.Run("only cpu fields with ignoreValues", func(t *testing.T) {
		validationReq := &kubewarden_protocol.ValidationRequest{
			Settings: []byte(
				`{"cpu":{"maxLimit": "1","defaultRequest": "1", "defaultLimit": "1", "ignoreValues": true}}`,
			),
		}
		settings, err := NewSettingsFromValidationReq(validationReq)
		if err != nil {
			t.Fatalf("Unexpected error %+v", err)
		}
		if settings.Memory != nil {
			t.Fatal("memory settings should be null")
		}
		checkSettingsValues(t, settings.CPU, "1", "1", "1", true)
	})
	t.Run("both cpu and memory fields with ignoreValues", func(t *testing.T) {
		validationReq := &kubewarden_protocol.ValidationRequest{
			Settings: []byte(`{"cpu":{"ignoreValues": true}, "memory":{"ignoreValues": true}}`),
		}
		settings, err := NewSettingsFromValidationReq(validationReq)
		if err != nil {
			t.Fatalf("Unexpected error %+v", err)
		}
		checkSettingsValues(t, settings.CPU, "0", "0", "0", true)
		checkSettingsValues(t, settings.Memory, "0", "0", "0", true)
	})
}
