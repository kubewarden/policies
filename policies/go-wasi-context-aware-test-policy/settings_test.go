package main

import (
	"encoding/json"
	"testing"

	mapset "github.com/deckarep/golang-set/v2"
	kubewardenProtocol "github.com/kubewarden/policy-sdk-go/protocol"
)

func TestValidateSettings(t *testing.T) {
	const ccCenterAnnotation = "cc-center"
	const marketingValue = "marketing"

	cases := []struct {
		name                 string
		requiredAnnotations  map[string]string
		forbiddenAnnotations mapset.Set[string]
		isValid              bool
	}{
		{
			"empty",
			map[string]string{},
			mapset.NewSet[string](),
			true,
		},
		{
			"only required annotations",
			map[string]string{
				ccCenterAnnotation: marketingValue,
			},
			mapset.NewSet[string](),
			true,
		},
		{
			"only forbidden annotations",
			map[string]string{},
			mapset.NewSet[string]("priority"),
			true,
		},
		{
			"no contradictions",
			map[string]string{
				ccCenterAnnotation: marketingValue,
			},
			mapset.NewSet[string]("priority"),
			true,
		},
		{
			"contradictions",
			map[string]string{
				ccCenterAnnotation: marketingValue,
			},
			mapset.NewSet[string](ccCenterAnnotation),
			false,
		},
	}

	for _, testCase := range cases {
		t.Run(testCase.name, func(t *testing.T) {
			settings := Settings{
				RequiredAnnotations:  testCase.requiredAnnotations,
				ForbiddenAnnotations: testCase.forbiddenAnnotations,
			}
			settingsJSON, err := json.Marshal(&settings)
			if err != nil {
				t.Errorf("cannot marshal settings: %v", err)
			}

			responseJSON, err := validateSettings(settingsJSON)
			if err != nil {
				t.Errorf("unexpected error: %v", err)
			}
			var response kubewardenProtocol.SettingsValidationResponse
			err = json.Unmarshal(responseJSON, &response)
			if err != nil {
				t.Errorf("cannot unmarshal response: %v", err)
			}

			if response.Valid != testCase.isValid {
				t.Errorf(
					"didn't get the expected validation outcome, %v was expected, got %v instead",
					testCase.isValid, response.Valid)
				if response.Message != nil {
					t.Errorf(
						"validation message: %s",
						*response.Message)
				}
			}
		})
	}
}
