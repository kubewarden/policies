package main

import (
	"fmt"
	"strings"
	"testing"

	"github.com/google/go-cmp/cmp"
	"github.com/kubewarden/container-resources-policy/resource"
	corev1 "github.com/kubewarden/k8s-objects/api/core/v1"
	apimachinery_pkg_api_resource "github.com/kubewarden/k8s-objects/apimachinery/pkg/api/resource"
)

// containerLimitsTestCase describes a single TestContainerIsRequiredToHaveLimits test case.
type containerLimitsTestCase struct {
	name                        string
	container                   corev1.Container
	settings                    Settings
	expectedResouceRequirements *corev1.ResourceRequirements
	shouldMutate                bool
	expectedErrorMsg            string
}

func TestContainerIsRequiredToHaveLimits(t *testing.T) {
	oneCore := resource.MustParse("1")
	oneGi := resource.MustParse("1Gi")
	oneCoreCPUQuantity := apimachinery_pkg_api_resource.Quantity("1")
	oneGiMemoryQuantity := apimachinery_pkg_api_resource.Quantity("1Gi")
	twoCore := resource.MustParse("2")
	twoGi := resource.MustParse("2Gi")
	twoCoreCPUQuantity := apimachinery_pkg_api_resource.Quantity("2")
	twoGiMemoryQuantity := apimachinery_pkg_api_resource.Quantity("2Gi")
	fourCore := apimachinery_pkg_api_resource.Quantity("4")
	tenCore := apimachinery_pkg_api_resource.Quantity("10")

	tests := []containerLimitsTestCase{
		{
			"no resources requests and limits defined",
			corev1.Container{},
			Settings{
				CPU: &ResourceConfiguration{
					DefaultRequest: oneCore,
					DefaultLimit:   oneCore,
				},
				Memory: &ResourceConfiguration{
					DefaultRequest: oneGi,
					DefaultLimit:   oneGi,
				},
				IgnoreImages: []string{},
			},
			&corev1.ResourceRequirements{
				Limits: map[string]*apimachinery_pkg_api_resource.Quantity{
					"cpu":    &oneCoreCPUQuantity,
					"memory": &oneGiMemoryQuantity,
				},
				Requests: map[string]*apimachinery_pkg_api_resource.Quantity{
					"cpu":    &oneCoreCPUQuantity,
					"memory": &oneGiMemoryQuantity,
				},
			}, true, "",
		},
		{
			"no memory limit",
			corev1.Container{
				Resources: &corev1.ResourceRequirements{
					Limits: map[string]*apimachinery_pkg_api_resource.Quantity{
						"cpu": &oneCoreCPUQuantity,
					},
					Requests: map[string]*apimachinery_pkg_api_resource.Quantity{
						"cpu":    &oneCoreCPUQuantity,
						"memory": &oneGiMemoryQuantity,
					},
				},
			},
			Settings{
				CPU: &ResourceConfiguration{
					DefaultLimit:   oneCore,
					DefaultRequest: oneCore,
					MaxLimit:       oneCore,
				},
				Memory: &ResourceConfiguration{
					DefaultLimit:   oneGi,
					DefaultRequest: oneGi,
					MaxLimit:       oneGi,
				},
				IgnoreImages: []string{},
			},
			&corev1.ResourceRequirements{
				Limits: map[string]*apimachinery_pkg_api_resource.Quantity{
					"cpu":    &oneCoreCPUQuantity,
					"memory": &oneGiMemoryQuantity,
				},
				Requests: map[string]*apimachinery_pkg_api_resource.Quantity{
					"cpu":    &oneCoreCPUQuantity,
					"memory": &oneGiMemoryQuantity,
				},
			}, true, "",
		},
		{
			"no cpu limit",
			corev1.Container{
				Resources: &corev1.ResourceRequirements{
					Limits: map[string]*apimachinery_pkg_api_resource.Quantity{
						"memory": &oneGiMemoryQuantity,
					},
					Requests: map[string]*apimachinery_pkg_api_resource.Quantity{
						"cpu":    &oneCoreCPUQuantity,
						"memory": &oneGiMemoryQuantity,
					},
				},
			},

			Settings{
				CPU: &ResourceConfiguration{
					DefaultLimit:   oneCore,
					DefaultRequest: oneCore,
					MaxLimit:       oneCore,
				},
				Memory: &ResourceConfiguration{
					DefaultLimit:   oneGi,
					DefaultRequest: oneGi,
					MaxLimit:       oneGi,
				},
				IgnoreImages: []string{},
			},
			&corev1.ResourceRequirements{
				Limits: map[string]*apimachinery_pkg_api_resource.Quantity{
					"cpu":    &oneCoreCPUQuantity,
					"memory": &oneGiMemoryQuantity,
				},
				Requests: map[string]*apimachinery_pkg_api_resource.Quantity{
					"cpu":    &oneCoreCPUQuantity,
					"memory": &oneGiMemoryQuantity,
				},
			}, true, "",
		},
		{
			"all limits within the expected range",
			corev1.Container{
				Resources: &corev1.ResourceRequirements{
					Limits: map[string]*apimachinery_pkg_api_resource.Quantity{
						"cpu":    &oneCoreCPUQuantity,
						"memory": &oneGiMemoryQuantity,
					},
					Requests: map[string]*apimachinery_pkg_api_resource.Quantity{
						"cpu":    &oneCoreCPUQuantity,
						"memory": &oneGiMemoryQuantity,
					},
				},
			},
			Settings{
				CPU: &ResourceConfiguration{
					MinRequest:     oneCore,
					MaxLimit:       twoCore,
					DefaultLimit:   twoCore,
					DefaultRequest: twoCore,
				},
				Memory: &ResourceConfiguration{
					MinRequest:     oneGi,
					MaxLimit:       twoGi,
					DefaultLimit:   twoGi,
					DefaultRequest: twoGi,
				},
			},
			&corev1.ResourceRequirements{
				Limits: map[string]*apimachinery_pkg_api_resource.Quantity{
					"cpu":    &oneCoreCPUQuantity,
					"memory": &oneGiMemoryQuantity,
				},
				Requests: map[string]*apimachinery_pkg_api_resource.Quantity{
					"cpu":    &oneCoreCPUQuantity,
					"memory": &oneGiMemoryQuantity,
				},
			}, false, "",
		},
		{"cpu limit exceeding the expected range", corev1.Container{
			Resources: &corev1.ResourceRequirements{
				Limits: map[string]*apimachinery_pkg_api_resource.Quantity{
					"cpu":    &twoCoreCPUQuantity,
					"memory": &oneGiMemoryQuantity,
				},
				Requests: map[string]*apimachinery_pkg_api_resource.Quantity{
					"cpu":    &oneCoreCPUQuantity,
					"memory": &oneGiMemoryQuantity,
				},
			},
		}, Settings{
			CPU: &ResourceConfiguration{
				DefaultLimit:   oneCore,
				DefaultRequest: oneCore,
				MaxLimit:       oneCore,
			},
			Memory: &ResourceConfiguration{
				DefaultLimit:   oneGi,
				DefaultRequest: oneGi,
				MaxLimit:       oneGi,
			},
		}, &corev1.ResourceRequirements{
			Limits: map[string]*apimachinery_pkg_api_resource.Quantity{
				"cpu":    &twoCoreCPUQuantity,
				"memory": &oneGiMemoryQuantity,
			},
			Requests: map[string]*apimachinery_pkg_api_resource.Quantity{
				"cpu":    &oneCoreCPUQuantity,
				"memory": &oneGiMemoryQuantity,
			},
		}, false, "cpu limit '2' exceeds the max allowed value '1'"},
		{"memory limit exceeding the expected range", corev1.Container{
			Resources: &corev1.ResourceRequirements{
				Limits: map[string]*apimachinery_pkg_api_resource.Quantity{
					"cpu":    &oneCoreCPUQuantity,
					"memory": &twoGiMemoryQuantity,
				},
				Requests: make(map[string]*apimachinery_pkg_api_resource.Quantity),
			},
		}, Settings{
			CPU: &ResourceConfiguration{
				DefaultLimit:   oneCore,
				DefaultRequest: oneCore,
				MaxLimit:       oneCore,
			},
			Memory: &ResourceConfiguration{
				DefaultLimit:   oneGi,
				DefaultRequest: oneGi,
				MaxLimit:       oneGi,
			},
		}, &corev1.ResourceRequirements{
			Limits: map[string]*apimachinery_pkg_api_resource.Quantity{
				"cpu":    &oneCoreCPUQuantity,
				"memory": &twoGiMemoryQuantity,
			},
			Requests: make(map[string]*apimachinery_pkg_api_resource.Quantity),
		}, false, "memory limit '2Gi' exceeds the max allowed value '1Gi'"},
		{"cpu request not matching min request", corev1.Container{
			Resources: &corev1.ResourceRequirements{
				Limits: map[string]*apimachinery_pkg_api_resource.Quantity{
					"cpu":    &twoCoreCPUQuantity,
					"memory": &oneGiMemoryQuantity,
				},
				Requests: map[string]*apimachinery_pkg_api_resource.Quantity{
					"cpu":    &oneCoreCPUQuantity,
					"memory": &oneGiMemoryQuantity,
				},
			},
		}, Settings{
			CPU: &ResourceConfiguration{
				DefaultLimit:   twoCore,
				DefaultRequest: twoCore,
				MinRequest:     twoCore,
				MaxLimit:       twoCore,
			},
			Memory: &ResourceConfiguration{
				DefaultLimit:   twoGi,
				DefaultRequest: twoGi,
				MaxLimit:       twoGi,
			},
		}, &corev1.ResourceRequirements{
			Limits: map[string]*apimachinery_pkg_api_resource.Quantity{
				"cpu":    &twoCoreCPUQuantity,
				"memory": &oneGiMemoryQuantity,
			},
			Requests: map[string]*apimachinery_pkg_api_resource.Quantity{
				"cpu":    &oneCoreCPUQuantity,
				"memory": &oneGiMemoryQuantity,
			},
		}, false, "cpu request '1' doesn't reach the min allowed value '2'"},

		{
			"no memory request",
			corev1.Container{
				Resources: &corev1.ResourceRequirements{
					Limits: map[string]*apimachinery_pkg_api_resource.Quantity{
						"cpu":    &oneCoreCPUQuantity,
						"memory": &oneGiMemoryQuantity,
					},
					Requests: map[string]*apimachinery_pkg_api_resource.Quantity{
						"cpu": &oneCoreCPUQuantity,
					},
				},
			},
			Settings{
				CPU: &ResourceConfiguration{
					DefaultLimit:   oneCore,
					DefaultRequest: oneCore,
					MaxLimit:       oneCore,
				},
				Memory: &ResourceConfiguration{
					DefaultLimit:   oneGi,
					DefaultRequest: oneGi,
					MaxLimit:       oneGi,
				},
				IgnoreImages: []string{},
			},
			&corev1.ResourceRequirements{
				Limits: map[string]*apimachinery_pkg_api_resource.Quantity{
					"cpu":    &oneCoreCPUQuantity,
					"memory": &oneGiMemoryQuantity,
				},
				Requests: map[string]*apimachinery_pkg_api_resource.Quantity{
					"cpu":    &oneCoreCPUQuantity,
					"memory": &oneGiMemoryQuantity,
				},
			}, true, "",
		},
		{
			"no cpu request",
			corev1.Container{
				Resources: &corev1.ResourceRequirements{
					Limits: map[string]*apimachinery_pkg_api_resource.Quantity{
						"cpu":    &oneCoreCPUQuantity,
						"memory": &oneGiMemoryQuantity,
					},
					Requests: map[string]*apimachinery_pkg_api_resource.Quantity{
						"memory": &oneGiMemoryQuantity,
					},
				},
			},

			Settings{
				CPU: &ResourceConfiguration{
					DefaultLimit:   oneCore,
					DefaultRequest: oneCore,
					MaxLimit:       oneCore,
				},
				Memory: &ResourceConfiguration{
					DefaultLimit:   oneGi,
					DefaultRequest: oneGi,
					MaxLimit:       oneGi,
				},
				IgnoreImages: []string{},
			},
			&corev1.ResourceRequirements{
				Limits: map[string]*apimachinery_pkg_api_resource.Quantity{
					"cpu":    &oneCoreCPUQuantity,
					"memory": &oneGiMemoryQuantity,
				},
				Requests: map[string]*apimachinery_pkg_api_resource.Quantity{
					"cpu":    &oneCoreCPUQuantity,
					"memory": &oneGiMemoryQuantity,
				},
			}, true, "",
		},

		{
			"requested resources are not validated when user define them",
			corev1.Container{
				Resources: &corev1.ResourceRequirements{
					Limits: map[string]*apimachinery_pkg_api_resource.Quantity{
						"cpu":    &oneCoreCPUQuantity,
						"memory": &oneGiMemoryQuantity,
					},
					Requests: map[string]*apimachinery_pkg_api_resource.Quantity{
						"cpu":    &twoCoreCPUQuantity,
						"memory": &twoGiMemoryQuantity,
					},
				},
			},
			Settings{
				CPU: &ResourceConfiguration{
					DefaultLimit:   oneCore,
					DefaultRequest: oneCore,
					MaxLimit:       oneCore,
				},
				Memory: &ResourceConfiguration{
					DefaultLimit:   oneGi,
					DefaultRequest: oneGi,
					MaxLimit:       oneGi,
				},
				IgnoreImages: []string{},
			},
			&corev1.ResourceRequirements{
				Limits: map[string]*apimachinery_pkg_api_resource.Quantity{
					"cpu":    &oneCoreCPUQuantity,
					"memory": &oneGiMemoryQuantity,
				},
				Requests: map[string]*apimachinery_pkg_api_resource.Quantity{
					"cpu":    &twoCoreCPUQuantity,
					"memory": &twoGiMemoryQuantity,
				},
			}, false, "",
		},
		{
			"resources with nil limits and requests",
			corev1.Container{
				Resources: &corev1.ResourceRequirements{
					Limits: map[string]*apimachinery_pkg_api_resource.Quantity{
						"cpu":    nil,
						"memory": nil,
					},
					Requests: map[string]*apimachinery_pkg_api_resource.Quantity{
						"cpu":    nil,
						"memory": nil,
					},
				},
			},
			Settings{
				CPU: &ResourceConfiguration{
					DefaultLimit:   oneCore,
					DefaultRequest: oneCore,
					MaxLimit:       oneCore,
				},
				Memory: &ResourceConfiguration{
					DefaultLimit:   oneGi,
					DefaultRequest: oneGi,
					MaxLimit:       oneGi,
				},
				IgnoreImages: []string{},
			},
			&corev1.ResourceRequirements{
				Requests: map[string]*apimachinery_pkg_api_resource.Quantity{
					"cpu":    &oneCoreCPUQuantity,
					"memory": &oneGiMemoryQuantity,
				},
				Limits: map[string]*apimachinery_pkg_api_resource.Quantity{
					"cpu":    &oneCoreCPUQuantity,
					"memory": &oneGiMemoryQuantity,
				},
			}, true, "",
		},
		{"cpu exceeds limit while ignore memory values", corev1.Container{
			Image: "image1:latest",
			Resources: &corev1.ResourceRequirements{
				Limits: map[string]*apimachinery_pkg_api_resource.Quantity{
					"memory": &twoGiMemoryQuantity,
					"cpu":    &twoCoreCPUQuantity,
				},
				Requests: map[string]*apimachinery_pkg_api_resource.Quantity{
					"memory": &twoGiMemoryQuantity,
					"cpu":    &twoCoreCPUQuantity,
				},
			},
		}, Settings{
			CPU: &ResourceConfiguration{
				DefaultLimit:   oneCore,
				DefaultRequest: oneCore,
				MaxLimit:       oneCore,
			},
			Memory: &ResourceConfiguration{
				IgnoreValues:   true,
				DefaultLimit:   oneGi,
				DefaultRequest: oneGi,
				MaxLimit:       oneGi,
			},
		}, &corev1.ResourceRequirements{
			Limits: map[string]*apimachinery_pkg_api_resource.Quantity{
				"memory": &twoGiMemoryQuantity,
				"cpu":    &twoCoreCPUQuantity,
			},
			Requests: map[string]*apimachinery_pkg_api_resource.Quantity{
				"memory": &twoGiMemoryQuantity,
				"cpu":    &twoCoreCPUQuantity,
			},
		}, false, "cpu limit '2' exceeds the max allowed value '1'"},
		{"memory exceeds limit while ignore cpu values", corev1.Container{
			Resources: &corev1.ResourceRequirements{
				Limits: map[string]*apimachinery_pkg_api_resource.Quantity{
					"memory": &twoGiMemoryQuantity,
					"cpu":    &twoCoreCPUQuantity,
				},
				Requests: map[string]*apimachinery_pkg_api_resource.Quantity{
					"memory": &twoGiMemoryQuantity,
					"cpu":    &twoCoreCPUQuantity,
				},
			},
		}, Settings{
			CPU: &ResourceConfiguration{
				IgnoreValues:   true,
				DefaultLimit:   oneCore,
				DefaultRequest: oneCore,
				MaxLimit:       oneCore,
			},
			Memory: &ResourceConfiguration{
				DefaultLimit:   oneGi,
				DefaultRequest: oneGi,
				MaxLimit:       oneGi,
			},
		}, &corev1.ResourceRequirements{
			Limits: map[string]*apimachinery_pkg_api_resource.Quantity{
				"memory": &twoGiMemoryQuantity,
				"cpu":    &twoCoreCPUQuantity,
			},
			Requests: map[string]*apimachinery_pkg_api_resource.Quantity{
				"memory": &twoGiMemoryQuantity,
				"cpu":    &twoCoreCPUQuantity,
			},
		}, false, "memory limit '2Gi' exceeds the max allowed value '1Gi'"},
		{
			"no memory limit and request greater then the default limit value",
			corev1.Container{
				Resources: &corev1.ResourceRequirements{
					Limits: map[string]*apimachinery_pkg_api_resource.Quantity{
						"cpu": &oneCoreCPUQuantity,
					},
					Requests: map[string]*apimachinery_pkg_api_resource.Quantity{
						"cpu":    &oneCoreCPUQuantity,
						"memory": &twoGiMemoryQuantity,
					},
				},
			},

			Settings{
				CPU: &ResourceConfiguration{
					DefaultLimit:   oneCore,
					DefaultRequest: oneCore,
					MaxLimit:       oneCore,
				},
				Memory: &ResourceConfiguration{
					DefaultLimit:   oneGi,
					DefaultRequest: oneGi,
					MaxLimit:       oneGi,
				},
				IgnoreImages: []string{},
			},
			&corev1.ResourceRequirements{
				Limits: map[string]*apimachinery_pkg_api_resource.Quantity{
					"cpu":    &oneCoreCPUQuantity,
					"memory": &oneGiMemoryQuantity,
				},
				Requests: map[string]*apimachinery_pkg_api_resource.Quantity{
					"cpu":    &oneCoreCPUQuantity,
					"memory": &twoGiMemoryQuantity,
				},
			}, false, "memory limit '1Gi' is less than the requested '2Gi' value",
		},
		{
			"no cpu limit and request greater then the default limit value",
			corev1.Container{
				Resources: &corev1.ResourceRequirements{
					Limits: map[string]*apimachinery_pkg_api_resource.Quantity{
						"memory": &oneGiMemoryQuantity,
					},
					Requests: map[string]*apimachinery_pkg_api_resource.Quantity{
						"cpu":    &twoCoreCPUQuantity,
						"memory": &oneGiMemoryQuantity,
					},
				},
			},

			Settings{
				CPU: &ResourceConfiguration{
					DefaultLimit:   oneCore,
					DefaultRequest: oneCore,
					MaxLimit:       oneCore,
				},
				Memory: &ResourceConfiguration{
					DefaultLimit:   oneGi,
					DefaultRequest: oneGi,
					MaxLimit:       oneGi,
				},
				IgnoreImages: []string{},
			},
			&corev1.ResourceRequirements{
				Limits: map[string]*apimachinery_pkg_api_resource.Quantity{
					"memory": &oneGiMemoryQuantity,
					"cpu":    &oneCoreCPUQuantity,
				},
				Requests: map[string]*apimachinery_pkg_api_resource.Quantity{
					"cpu":    &twoCoreCPUQuantity,
					"memory": &oneGiMemoryQuantity,
				},
			}, false, "cpu limit '1' is less than the requested '2' value",
		},
		{
			"cpu limit below minLimit",
			corev1.Container{
				Resources: &corev1.ResourceRequirements{
					Limits: map[string]*apimachinery_pkg_api_resource.Quantity{
						"cpu": &oneCoreCPUQuantity,
					},
					Requests: map[string]*apimachinery_pkg_api_resource.Quantity{
						"cpu": &oneCoreCPUQuantity,
					},
				},
			},
			Settings{
				CPU: &ResourceConfiguration{
					MinLimit:       twoCore,
					MaxLimit:       twoCore,
					DefaultLimit:   twoCore,
					DefaultRequest: oneCore,
				},
				IgnoreImages: []string{},
			},
			&corev1.ResourceRequirements{
				Limits: map[string]*apimachinery_pkg_api_resource.Quantity{
					"cpu": &oneCoreCPUQuantity,
				},
				Requests: map[string]*apimachinery_pkg_api_resource.Quantity{
					"cpu": &oneCoreCPUQuantity,
				},
			},
			false,
			"cpu limit '1' doesn't reach the min allowed value '2'",
		},
		{
			"cpu request exceeds maxRequest",
			corev1.Container{
				Resources: &corev1.ResourceRequirements{
					Requests: map[string]*apimachinery_pkg_api_resource.Quantity{
						"cpu": &twoCoreCPUQuantity,
					},
				},
			},
			Settings{
				CPU: &ResourceConfiguration{
					MaxRequest: oneCore,
				},
			},
			&corev1.ResourceRequirements{
				Requests: map[string]*apimachinery_pkg_api_resource.Quantity{
					"cpu": &twoCoreCPUQuantity,
				},
				Limits: map[string]*apimachinery_pkg_api_resource.Quantity{},
			},
			false,
			"cpu request '2' exceeds the max allowed value '1'",
		},
		{
			"memory request exceeds maxRequest",
			corev1.Container{
				Resources: &corev1.ResourceRequirements{
					Requests: map[string]*apimachinery_pkg_api_resource.Quantity{
						"memory": &twoGiMemoryQuantity,
					},
				},
			},
			Settings{
				Memory: &ResourceConfiguration{
					MaxRequest: oneGi,
				},
			},
			&corev1.ResourceRequirements{
				Requests: map[string]*apimachinery_pkg_api_resource.Quantity{
					"memory": &twoGiMemoryQuantity,
				},
				Limits: map[string]*apimachinery_pkg_api_resource.Quantity{},
			},
			false,
			"memory request '2Gi' exceeds the max allowed value '1Gi'",
		},
		{
			"cpu limit below minLimit",
			corev1.Container{
				Resources: &corev1.ResourceRequirements{
					Limits: map[string]*apimachinery_pkg_api_resource.Quantity{
						"cpu": &oneCoreCPUQuantity,
					},
				},
			},
			Settings{
				CPU: &ResourceConfiguration{
					MinLimit: twoCore,
				},
			},
			&corev1.ResourceRequirements{
				Requests: map[string]*apimachinery_pkg_api_resource.Quantity{},
				Limits: map[string]*apimachinery_pkg_api_resource.Quantity{
					"cpu": &oneCoreCPUQuantity,
				},
			},
			false,
			"cpu limit '1' doesn't reach the min allowed value '2'",
		},
		{
			"request below minLimit but within maxRequest should be accepted",
			corev1.Container{
				Resources: &corev1.ResourceRequirements{
					Requests: map[string]*apimachinery_pkg_api_resource.Quantity{
						"cpu": &fourCore,
					},
					Limits: map[string]*apimachinery_pkg_api_resource.Quantity{
						"cpu": &tenCore,
					},
				},
			},
			Settings{
				CPU: &ResourceConfiguration{
					MaxRequest: resource.MustParse("6"),
					MinLimit:   resource.MustParse("6"),
				},
			},
			&corev1.ResourceRequirements{
				Requests: map[string]*apimachinery_pkg_api_resource.Quantity{
					"cpu": &fourCore,
				},
				Limits: map[string]*apimachinery_pkg_api_resource.Quantity{
					"cpu": &tenCore,
				},
			},
			false,
			"",
		},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			runContainerLimitsTestCase(t, &test)
		})
	}
}

// runContainerLimitsTestCase executes a single TestContainerIsRequiredToHaveLimits test case.
func runContainerLimitsTestCase(t *testing.T, test *containerLimitsTestCase) {
	mutated, err := validateAndAdjustContainer(&test.container, &test.settings)
	if err != nil && len(test.expectedErrorMsg) == 0 {
		t.Fatalf("unexpected error: %q", err)
	}
	if len(test.expectedErrorMsg) > 0 {
		if err == nil {
			t.Fatalf(
				"expected error message with string '%s'. But no error has been returned",
				test.expectedErrorMsg,
			)
		}
		if !strings.Contains(err.Error(), test.expectedErrorMsg) {
			t.Fatalf(
				"invalid error message. Expected the string '%s' in the error. Got '%s'",
				test.expectedErrorMsg,
				err.Error(),
			)
		}
	}
	if mutated != test.shouldMutate {
		t.Fatalf(
			"validation function does not report mutation flag correctly. Got: %t, expected: %t",
			mutated,
			test.shouldMutate,
		)
	}
	if diff := cmp.Diff(test.container.Resources, test.expectedResouceRequirements); diff != "" {
		t.Fatalf("%s", diff)
	}
}

func TestIgnoreValues(t *testing.T) {
	oneCore := resource.MustParse("1")
	oneGi := resource.MustParse("1Gi")
	oneCoreCPUQuantity := apimachinery_pkg_api_resource.Quantity("1")
	oneGiMemoryQuantity := apimachinery_pkg_api_resource.Quantity("1Gi")
	twoCoreCPUQuantity := apimachinery_pkg_api_resource.Quantity("2")
	tests := []struct {
		name                  string
		container             corev1.Container
		settings              Settings
		expectedResouceLimits *corev1.ResourceRequirements
		expectedErrorMsg      string
	}{
		{
			"memory resources requests and limits defined and ignore cpu",
			corev1.Container{
				Image: "image1:latest",
				Resources: &corev1.ResourceRequirements{
					Limits: map[string]*apimachinery_pkg_api_resource.Quantity{
						"cpu":    &oneCoreCPUQuantity,
						"memory": &oneGiMemoryQuantity,
					},
					Requests: map[string]*apimachinery_pkg_api_resource.Quantity{
						"cpu":    &oneCoreCPUQuantity,
						"memory": &oneGiMemoryQuantity,
					},
				},
			},
			Settings{
				CPU: &ResourceConfiguration{
					IgnoreValues: true,
				},
				Memory: &ResourceConfiguration{
					DefaultLimit:   oneGi,
					DefaultRequest: oneGi,
					MaxLimit:       oneGi,
				},
				IgnoreImages: []string{"image1:latest"},
			},
			&corev1.ResourceRequirements{
				Limits: map[string]*apimachinery_pkg_api_resource.Quantity{
					"cpu":    &oneCoreCPUQuantity,
					"memory": &oneGiMemoryQuantity,
				},
				Requests: map[string]*apimachinery_pkg_api_resource.Quantity{
					"cpu":    &oneCoreCPUQuantity,
					"memory": &oneGiMemoryQuantity,
				},
			}, "",
		},
		{"cpu resources requests and limits defined and ignore memory", corev1.Container{
			Image: "image1:latest",
			Resources: &corev1.ResourceRequirements{
				Limits: map[string]*apimachinery_pkg_api_resource.Quantity{
					"cpu":    &oneCoreCPUQuantity,
					"memory": &oneGiMemoryQuantity,
				},
				Requests: map[string]*apimachinery_pkg_api_resource.Quantity{
					"cpu":    &oneCoreCPUQuantity,
					"memory": &oneGiMemoryQuantity,
				},
			},
		}, Settings{
			CPU: &ResourceConfiguration{
				DefaultLimit:   oneCore,
				DefaultRequest: oneCore,
				MaxLimit:       oneCore,
				IgnoreValues:   true,
			},
			Memory: &ResourceConfiguration{
				IgnoreValues: true,
			},
		}, &corev1.ResourceRequirements{
			Limits: map[string]*apimachinery_pkg_api_resource.Quantity{
				"cpu":    &oneCoreCPUQuantity,
				"memory": &oneGiMemoryQuantity,
			},
			Requests: map[string]*apimachinery_pkg_api_resource.Quantity{
				"cpu":    &oneCoreCPUQuantity,
				"memory": &oneGiMemoryQuantity,
			},
		}, ""},
		{
			"container with no resources defined and ignore values",
			corev1.Container{},
			Settings{
				CPU: &ResourceConfiguration{
					IgnoreValues: true,
				},
				Memory: &ResourceConfiguration{
					IgnoreValues: true,
				},
			},
			&corev1.ResourceRequirements{},
			"container does not have any resource limits",
		},
		{"container with missing cpu values and ignore cpu values", corev1.Container{
			Image: "image1:latest",
			Resources: &corev1.ResourceRequirements{
				Limits: map[string]*apimachinery_pkg_api_resource.Quantity{
					"memory": &oneGiMemoryQuantity,
				},
				Requests: map[string]*apimachinery_pkg_api_resource.Quantity{
					"memory": &oneGiMemoryQuantity,
				},
			},
		}, Settings{
			CPU: &ResourceConfiguration{
				IgnoreValues: true,
			},
			Memory: &ResourceConfiguration{
				DefaultLimit:   oneGi,
				DefaultRequest: oneGi,
				MaxLimit:       oneGi,
			},
		}, &corev1.ResourceRequirements{
			Limits: map[string]*apimachinery_pkg_api_resource.Quantity{
				"memory": &oneGiMemoryQuantity,
			},
			Requests: map[string]*apimachinery_pkg_api_resource.Quantity{
				"memory": &oneGiMemoryQuantity,
			},
		}, "container does not have a cpu limit"},
		{"container with missing memory values and ignore memory values", corev1.Container{
			Image: "image1:latest",
			Resources: &corev1.ResourceRequirements{
				Limits: map[string]*apimachinery_pkg_api_resource.Quantity{
					"cpu": &twoCoreCPUQuantity,
				},
				Requests: map[string]*apimachinery_pkg_api_resource.Quantity{
					"cpu": &oneCoreCPUQuantity,
				},
			},
		}, Settings{
			CPU: &ResourceConfiguration{
				DefaultLimit:   oneCore,
				DefaultRequest: oneCore,
				MaxLimit:       oneCore,
			},
			Memory: &ResourceConfiguration{
				IgnoreValues: true,
			},
		}, &corev1.ResourceRequirements{
			Limits: map[string]*apimachinery_pkg_api_resource.Quantity{
				"cpu": &twoCoreCPUQuantity,
			},
			Requests: map[string]*apimachinery_pkg_api_resource.Quantity{
				"cpu": &oneCoreCPUQuantity,
			},
		}, "container does not have a memory limit"},
		{"container missing memory requests values and ignore memory values", corev1.Container{
			Image: "image1:latest",
			Resources: &corev1.ResourceRequirements{
				Limits: map[string]*apimachinery_pkg_api_resource.Quantity{
					"cpu":    &oneCoreCPUQuantity,
					"memory": &oneGiMemoryQuantity,
				},
				Requests: map[string]*apimachinery_pkg_api_resource.Quantity{
					"cpu": &oneCoreCPUQuantity,
				},
			},
		}, Settings{
			CPU: &ResourceConfiguration{
				DefaultLimit:   oneCore,
				DefaultRequest: oneCore,
				MaxLimit:       oneCore,
			},
			Memory: &ResourceConfiguration{
				IgnoreValues: true,
			},
		}, &corev1.ResourceRequirements{
			Limits: map[string]*apimachinery_pkg_api_resource.Quantity{
				"cpu":    &oneCoreCPUQuantity,
				"memory": &oneGiMemoryQuantity,
			},
			Requests: map[string]*apimachinery_pkg_api_resource.Quantity{
				"cpu": &oneCoreCPUQuantity,
			},
		}, "container does not have a memory request"},
		{"no memory settings", corev1.Container{
			Image: "image1:latest",
			Resources: &corev1.ResourceRequirements{
				Limits: map[string]*apimachinery_pkg_api_resource.Quantity{
					"cpu":    &oneCoreCPUQuantity,
					"memory": &oneGiMemoryQuantity,
				},
				Requests: map[string]*apimachinery_pkg_api_resource.Quantity{
					"cpu": &oneCoreCPUQuantity,
				},
			},
		}, Settings{
			CPU: &ResourceConfiguration{
				DefaultLimit:   oneCore,
				DefaultRequest: oneCore,
				MaxLimit:       oneCore,
			},
			Memory: nil,
		}, &corev1.ResourceRequirements{
			Limits: map[string]*apimachinery_pkg_api_resource.Quantity{
				"cpu":    &oneCoreCPUQuantity,
				"memory": &oneGiMemoryQuantity,
			},
			Requests: map[string]*apimachinery_pkg_api_resource.Quantity{
				"cpu": &oneCoreCPUQuantity,
			},
		}, ""},
		{"no cpu settings", corev1.Container{
			Image: "image1:latest",
			Resources: &corev1.ResourceRequirements{
				Limits: map[string]*apimachinery_pkg_api_resource.Quantity{
					"memory": &oneGiMemoryQuantity,
				},
				Requests: map[string]*apimachinery_pkg_api_resource.Quantity{
					"memory": &oneGiMemoryQuantity,
				},
			},
		}, Settings{
			CPU: nil,
			Memory: &ResourceConfiguration{
				DefaultLimit:   oneGi,
				DefaultRequest: oneGi,
				MaxLimit:       oneGi,
			},
		}, &corev1.ResourceRequirements{
			Limits: map[string]*apimachinery_pkg_api_resource.Quantity{
				"memory": &oneGiMemoryQuantity,
			},
			Requests: map[string]*apimachinery_pkg_api_resource.Quantity{
				"memory": &oneGiMemoryQuantity,
			},
		}, ""},
	}
	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			err := validateContainerCheckPresence(&test.container, &test.settings)
			if err != nil && len(test.expectedErrorMsg) == 0 {
				t.Fatalf("unexpected error: %q", err)
			}
			if len(test.expectedErrorMsg) > 0 {
				if err == nil {
					t.Fatalf(
						"expected error message with string '%s'. But no error has been returned",
						test.expectedErrorMsg,
					)
				}
				if !strings.Contains(err.Error(), test.expectedErrorMsg) {
					t.Errorf(
						"invalid error message. Expected the string '%s' in the error. Got '%s'",
						test.expectedErrorMsg,
						err.Error(),
					)
				}
			}
		})
	}
}

func TestIgnoreImageSettings(t *testing.T) {
	oneCore := resource.MustParse("1")
	oneGi := resource.MustParse("1Gi")
	oneCoreCPUQuantity := apimachinery_pkg_api_resource.Quantity("1")
	oneGiMemoryQuantity := apimachinery_pkg_api_resource.Quantity("1Gi")
	container1 := corev1.Container{
		Image: "image1:latest",
		Resources: &corev1.ResourceRequirements{
			Requests: map[string]*apimachinery_pkg_api_resource.Quantity{},
			Limits:   map[string]*apimachinery_pkg_api_resource.Quantity{},
		},
	}
	container2 := corev1.Container{
		Image: "image2:latest",
		Resources: &corev1.ResourceRequirements{
			Requests: map[string]*apimachinery_pkg_api_resource.Quantity{},
			Limits:   map[string]*apimachinery_pkg_api_resource.Quantity{},
		},
	}
	container3 := corev1.Container{
		Image: "image3:latest",
		Resources: &corev1.ResourceRequirements{
			Requests: map[string]*apimachinery_pkg_api_resource.Quantity{},
			Limits:   map[string]*apimachinery_pkg_api_resource.Quantity{},
		},
	}
	settings := Settings{
		CPU: &ResourceConfiguration{
			DefaultLimit:   oneCore,
			DefaultRequest: oneCore,
		},
		Memory: &ResourceConfiguration{
			DefaultLimit:   oneGi,
			DefaultRequest: oneGi,
		},
		IgnoreImages: []string{"image1:latest"},
	}
	podSpec := &corev1.PodSpec{
		Containers: []*corev1.Container{&container1, &container2, &container3},
	}
	mutate, err := validatePodSpec(podSpec, &settings)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if !mutate {
		t.Error("pod should be mutated")
	}
	expectedPodSpec := &corev1.PodSpec{
		Containers: []*corev1.Container{
			{
				Image: "image1:latest",
				Resources: &corev1.ResourceRequirements{
					Requests: map[string]*apimachinery_pkg_api_resource.Quantity{},
					Limits:   map[string]*apimachinery_pkg_api_resource.Quantity{},
				},
			},
			{
				Image: "image2:latest",
				Resources: &corev1.ResourceRequirements{
					Requests: map[string]*apimachinery_pkg_api_resource.Quantity{
						"cpu":    &oneCoreCPUQuantity,
						"memory": &oneGiMemoryQuantity,
					},
					Limits: map[string]*apimachinery_pkg_api_resource.Quantity{
						"cpu":    &oneCoreCPUQuantity,
						"memory": &oneGiMemoryQuantity,
					},
				},
			},
			{
				Image: "image3:latest",
				Resources: &corev1.ResourceRequirements{
					Requests: map[string]*apimachinery_pkg_api_resource.Quantity{
						"cpu":    &oneCoreCPUQuantity,
						"memory": &oneGiMemoryQuantity,
					},
					Limits: map[string]*apimachinery_pkg_api_resource.Quantity{
						"cpu":    &oneCoreCPUQuantity,
						"memory": &oneGiMemoryQuantity,
					},
				},
			},
		},
	}
	if diff := cmp.Diff(expectedPodSpec, podSpec); diff != "" {
		t.Errorf("invalid pod spec:\n %s", diff)
	}
}

func TestIgnoreImageWithNoTags(t *testing.T) {
	oneCore := resource.MustParse("1")
	oneGi := resource.MustParse("1Gi")
	oneCoreCPUQuantity := apimachinery_pkg_api_resource.Quantity("1")
	oneGiMemoryQuantity := apimachinery_pkg_api_resource.Quantity("1Gi")
	container1 := corev1.Container{
		Image: "othersimage:v1",
		Resources: &corev1.ResourceRequirements{
			Requests: map[string]*apimachinery_pkg_api_resource.Quantity{},
			Limits:   map[string]*apimachinery_pkg_api_resource.Quantity{},
		},
	}
	container2 := corev1.Container{
		Image: "othersimage:v2",
		Resources: &corev1.ResourceRequirements{
			Requests: map[string]*apimachinery_pkg_api_resource.Quantity{},
			Limits:   map[string]*apimachinery_pkg_api_resource.Quantity{},
		},
	}
	container3 := corev1.Container{
		Image: "myimage:latest",
		Resources: &corev1.ResourceRequirements{
			Requests: map[string]*apimachinery_pkg_api_resource.Quantity{},
			Limits:   map[string]*apimachinery_pkg_api_resource.Quantity{},
		},
	}
	settings := Settings{
		CPU: &ResourceConfiguration{
			DefaultLimit:   oneCore,
			DefaultRequest: oneCore,
		},
		Memory: &ResourceConfiguration{
			DefaultLimit:   oneGi,
			DefaultRequest: oneGi,
		},
		IgnoreImages: []string{"othersimage:*"},
	}
	podSpec := &corev1.PodSpec{
		Containers: []*corev1.Container{&container1, &container2, &container3},
	}
	mutate, err := validatePodSpec(podSpec, &settings)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if !mutate {
		t.Error("pod should be mutated")
	}
	expectedPodSpec := &corev1.PodSpec{
		Containers: []*corev1.Container{
			{
				Image: "othersimage:v1",
				Resources: &corev1.ResourceRequirements{
					Requests: map[string]*apimachinery_pkg_api_resource.Quantity{},
					Limits:   map[string]*apimachinery_pkg_api_resource.Quantity{},
				},
			},
			{
				Image: "othersimage:v2",
				Resources: &corev1.ResourceRequirements{
					Requests: map[string]*apimachinery_pkg_api_resource.Quantity{},
					Limits:   map[string]*apimachinery_pkg_api_resource.Quantity{},
				},
			},
			{
				Image: "myimage:latest",
				Resources: &corev1.ResourceRequirements{
					Requests: map[string]*apimachinery_pkg_api_resource.Quantity{
						"cpu":    &oneCoreCPUQuantity,
						"memory": &oneGiMemoryQuantity,
					},
					Limits: map[string]*apimachinery_pkg_api_resource.Quantity{
						"cpu":    &oneCoreCPUQuantity,
						"memory": &oneGiMemoryQuantity,
					},
				},
			},
		},
	}
	if diff := cmp.Diff(expectedPodSpec, podSpec); diff != "" {
		t.Errorf("invalid pod spec:\n %s", diff)
	}
}

func TestImageComparison(t *testing.T) {
	tests := []struct {
		image        string
		ignoreImages []string
		shouldSkip   bool
	}{
		{"image", []string{"image*"}, true},
		{"image", []string{"image:v1"}, false},
		{"image:v1", []string{"image"}, false},
		{"image", []string{"image"}, true},
		{"image:v1", []string{"image:v2"}, false},
		{"image:latest", []string{"image*"}, true},
		{"image:latest", []string{"image"}, false},
		{"image:latest", []string{"image:latest"}, true},
		{"image:latest", []string{"otherimage:latest"}, false},
		{"image:latest", []string{"otherimage"}, false},
		{"image:latest", []string{"image:*", "registry.k8s.io/pause*"}, true},
		{"registry.k8s.io/pause", []string{"image:*", "registry.k8s.io/pause*"}, true},
		{"fictional.registry.example:10443/imagename", []string{"imagename"}, false},
		{"fictional.registry.example:10443/imagename:v1.1.1", []string{"imagename"}, false},
		{"fictional.registry.example/imagename", []string{"imagename"}, false},
		{"fictional.registry.example/imagename:v1.1.1", []string{"imagename"}, false},
		{"fictional.registry.example:10443/imagename", []string{"imagename*"}, false},
		{"fictional.registry.example:10443/imagename:v1.1.1", []string{"imagename*"}, false},
		{"fictional.registry.example/imagename", []string{"imagename*"}, false},
		{"fictional.registry.example/imagename:v1.1.1", []string{"imagename*"}, false},
		{"reg.example.com/busybox:latest", []string{"reg.example.com/busybox:*"}, true},
		{"reg.example.com/busybox:1.23", []string{"reg.example.com/busybox:*"}, true},
		{"busybox:latest", []string{"reg.example.com/busybox:*"}, false},
		{"reg.example.io/busybox", []string{"reg.example.com/busybox:*"}, false},
	}
	for _, test := range tests {
		t.Run(fmt.Sprintf("%s in %v", test.image, test.ignoreImages), func(t *testing.T) {
			shouldSkip := shouldSkipContainer(test.image, test.ignoreImages)
			if shouldSkip != test.shouldSkip {
				t.Errorf("shouldValidateContainer returned %t, expected %t", shouldSkip, test.shouldSkip)
			}
		})
	}
}
