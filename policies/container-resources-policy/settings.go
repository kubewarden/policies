package main

import (
	"encoding/json"
	"errors"
	"fmt"

	"github.com/kubewarden/container-resources-policy/resource"
	kubewarden "github.com/kubewarden/policy-sdk-go"
	kubewarden_protocol "github.com/kubewarden/policy-sdk-go/protocol"
)

type ResourceConfiguration struct {
	MinLimit       resource.Quantity `json:"minLimit"`
	MaxLimit       resource.Quantity `json:"maxLimit"`
	MinRequest     resource.Quantity `json:"minRequest"`
	MaxRequest     resource.Quantity `json:"maxRequest"`
	DefaultRequest resource.Quantity `json:"defaultRequest"`
	DefaultLimit   resource.Quantity `json:"defaultLimit"`
	IgnoreValues   bool              `json:"ignoreValues,omitempty"`
}

type Settings struct {
	CPU          *ResourceConfiguration `json:"cpu,omitempty"`
	Memory       *ResourceConfiguration `json:"memory,omitempty"`
	IgnoreImages []string               `json:"ignoreImages,omitempty"`
}

type AllValuesAreZeroError struct{}

func (e AllValuesAreZeroError) Error() string {
	return "all the quantities must be defined"
}

func NewSettingsFromValidationReq(validationReq *kubewarden_protocol.ValidationRequest) (Settings, error) {
	settings := Settings{}
	err := json.Unmarshal(validationReq.Settings, &settings)
	return settings, err
}

func (s *Settings) shouldIgnoreCPUValues() bool {
	return s.CPU != nil && (s.CPU.IgnoreValues || (!s.CPU.IgnoreValues && s.CPU.allValuesAreZero()))
}

func (s *Settings) shouldIgnoreMemoryValues() bool {
	return s.Memory != nil && (s.Memory.IgnoreValues || (!s.Memory.IgnoreValues && s.Memory.allValuesAreZero()))
}

const (
	nameMaxLimit       = "max limit"
	nameMinLimit       = "min limit"
	nameDefaultLimit   = "default limit"
	nameMaxRequest     = "max request"
	nameMinRequest     = "min request"
	nameDefaultRequest = "default request"
)

func (r *ResourceConfiguration) valid() error {
	if r.allValuesAreZero() && !r.IgnoreValues {
		return AllValuesAreZeroError{}
	}

	// Core chain: minRequest <= defaultRequest <= maxRequest <= minLimit <= defaultLimit <= maxLimit
	// This enforces the constraint: limit >= request for all combinations
	// The chain ensures that any limit is always >= any request
	constraints := []struct {
		lesser      resource.Quantity
		lesserName  string
		greater     resource.Quantity
		greaterName string
	}{
		// Validate max limit relationships
		{r.DefaultLimit, nameDefaultLimit, r.MaxLimit, nameMaxLimit},
		{r.MinLimit, nameMinLimit, r.MaxLimit, nameMaxLimit},
		{r.MaxRequest, nameMaxRequest, r.MaxLimit, nameMaxLimit},
		{r.DefaultRequest, nameDefaultRequest, r.MaxLimit, nameMaxLimit},
		{r.MinRequest, nameMinRequest, r.MaxLimit, nameMaxLimit},
		// Validate default limit relationships
		{r.MinLimit, nameMinLimit, r.DefaultLimit, nameDefaultLimit},
		{r.MaxRequest, nameMaxRequest, r.DefaultLimit, nameDefaultLimit},
		{r.DefaultRequest, nameDefaultRequest, r.DefaultLimit, nameDefaultLimit},
		{r.MinRequest, nameMinRequest, r.DefaultLimit, nameDefaultLimit},
		// Validate min limit relationships
		{r.MaxRequest, nameMaxRequest, r.MinLimit, nameMinLimit},
		{r.DefaultRequest, nameDefaultRequest, r.MinLimit, nameMinLimit},
		{r.MinRequest, nameMinRequest, r.MinLimit, nameMinLimit},
		// Validate max request relationships
		{r.DefaultRequest, nameDefaultRequest, r.MaxRequest, nameMaxRequest},
		{r.MinRequest, nameMinRequest, r.MaxRequest, nameMaxRequest},
		// Validate default request relationships
		{r.MinRequest, nameMinRequest, r.DefaultRequest, nameDefaultRequest},
	}

	for _, constraint := range constraints {
		if !constraint.lesser.IsZero() && !constraint.greater.IsZero() &&
			constraint.lesser.Cmp(constraint.greater) > 0 {
			return fmt.Errorf(
				"%s: %s cannot be greater than %s: %s",
				constraint.lesserName,
				constraint.lesser.String(),
				constraint.greaterName,
				constraint.greater.String(),
			)
		}
	}

	return nil
}

func (r *ResourceConfiguration) allValuesAreZero() bool {
	return r.MaxLimit.IsZero() && r.DefaultLimit.IsZero() && r.DefaultRequest.IsZero() && r.MinRequest.IsZero() &&
		r.MinLimit.IsZero() &&
		r.MaxRequest.IsZero()
}

func (s *Settings) Valid() error {
	if s.CPU == nil && s.Memory == nil {
		return fmt.Errorf("no settings provided. At least one resource limit or request must be verified")
	}

	var cpuError, memoryError error
	if s.CPU != nil {
		cpuError = s.CPU.valid()
		if cpuError != nil {
			cpuError = errors.Join(fmt.Errorf("invalid cpu settings"), cpuError)
		}
	}
	if s.Memory != nil {
		memoryError = s.Memory.valid()
		if memoryError != nil {
			memoryError = errors.Join(fmt.Errorf("invalid memory settings"), memoryError)
		}
	}
	if cpuError != nil || memoryError != nil {
		// user want to validate only one type of resource. The other one should be ignored
		if (cpuError == nil && errors.Is(memoryError, AllValuesAreZeroError{})) ||
			(memoryError == nil && errors.Is(cpuError, AllValuesAreZeroError{})) {
			return nil
		}
		return errors.Join(cpuError, memoryError)
	}
	return nil
}

func validateSettings(payload []byte) ([]byte, error) {
	logger.Info("validating settings")
	settings := Settings{}
	err := json.Unmarshal(payload, &settings)
	if err != nil {
		return kubewarden.RejectSettings(kubewarden.Message(fmt.Sprintf("Provided settings are not valid: %v", err)))
	}

	err = settings.Valid()
	if err != nil {
		return kubewarden.RejectSettings(kubewarden.Message(fmt.Sprintf("Provided settings are not valid: %v", err)))
	}
	return kubewarden.AcceptSettings()
}
