package main

import (
	"encoding/json"
	"fmt"
	"regexp"
	"strings"

	mapset "github.com/deckarep/golang-set/v2"
	kubewarden "github.com/kubewarden/policy-sdk-go"
	kubewarden_protocol "github.com/kubewarden/policy-sdk-go/protocol"
)

// RegularExpression wraps the standard [regexp.Regexp] struct
// so that it implements marshalling and unmarshalling.
type RegularExpression struct {
	*regexp.Regexp
}

// CompileRegularExpression is a convenience method to build a regular expression.
func CompileRegularExpression(expr string) (*RegularExpression, error) {
	nativeRegExp, err := regexp.Compile(expr)
	if err != nil {
		return nil, err
	}
	return &RegularExpression{nativeRegExp}, nil
}

// UnmarshalText satisfies the [encoding.TextMarshaler] interface,
// also used by [json.Unmarshal].
func (r *RegularExpression) UnmarshalText(text []byte) error {
	nativeRegExp, err := regexp.Compile(string(text))
	if err != nil {
		return err
	}
	r.Regexp = nativeRegExp
	return nil
}

// MarshalText satisfies the [encoding.TextMarshaler] interface,
// also used by [json.Marshal].
//
//nolint:unparam // the error return value is required to satisfy the encoding.TextMarshaler interface
func (r *RegularExpression) MarshalText() ([]byte, error) {
	if r.Regexp != nil {
		return []byte(r.String()), nil
	}

	return nil, nil
}

type Settings struct {
	DeniedAnnotations      mapset.Set[string]            `json:"denied_annotations"`
	MandatoryAnnotations   mapset.Set[string]            `json:"mandatory_annotations"`
	ConstrainedAnnotations map[string]*RegularExpression `json:"constrained_annotations"`
}

// NewSettingsFromValidationReq builds a new Settings instance starting from a validation
// request payload:
//
//	{
//	   "request": ...,
//	   "settings": {
//	      "denied_annotations": [...],
//	      "mandatory_annotations": [...],
//	      "constrained_annotations": { ... }
//	   }
//	}
func NewSettingsFromValidationReq(validationRequest kubewarden_protocol.ValidationRequest) (Settings, error) {
	settings := Settings{}

	err := json.Unmarshal(validationRequest.Settings, &settings)
	if err != nil {
		return Settings{}, err
	}

	return settings, nil
}

func (s *Settings) Valid() (bool, error) {
	constrainedAnnotations := mapset.NewThreadUnsafeSet[string]()

	for annotations := range s.ConstrainedAnnotations {
		constrainedAnnotations.Add(annotations)
	}

	errors := []string{}

	constrainedAndDenied := constrainedAnnotations.Intersect(s.DeniedAnnotations)
	if constrainedAndDenied.Cardinality() != 0 {
		violations := constrainedAndDenied.ToSlice()
		errors = append(
			errors,
			fmt.Sprintf(
				"These annotations cannot be constrained and denied at the same time: %s",
				strings.Join(violations, ","),
			),
		)
	}

	mandatoryAndDenied := s.MandatoryAnnotations.Intersect(s.DeniedAnnotations)
	if mandatoryAndDenied.Cardinality() != 0 {
		violations := mandatoryAndDenied.ToSlice()
		errors = append(
			errors,
			fmt.Sprintf(
				"These annotations cannot be mandatory and denied at the same time: %s",
				strings.Join(violations, ","),
			),
		)
	}

	if len(errors) > 0 {
		return false, fmt.Errorf("%s", strings.Join(errors, "; "))
	}
	return true, nil
}

func (s *Settings) UnmarshalJSON(data []byte) error {
	// This is needed becaus golang-set v2.3.0 has a bug that prevents
	// the correct unmarshalling of ThreadUnsafeSet types.
	rawSettings := struct {
		DeniedAnnotations      []string                      `json:"denied_annotations"`
		MandatoryAnnotations   []string                      `json:"mandatory_annotations"`
		ConstrainedAnnotations map[string]*RegularExpression `json:"constrained_annotations"`
	}{}

	err := json.Unmarshal(data, &rawSettings)
	if err != nil {
		return err
	}

	s.DeniedAnnotations = mapset.NewThreadUnsafeSet[string](rawSettings.DeniedAnnotations...)
	s.MandatoryAnnotations = mapset.NewThreadUnsafeSet[string](rawSettings.MandatoryAnnotations...)
	s.ConstrainedAnnotations = rawSettings.ConstrainedAnnotations

	return nil
}

func validateSettings(payload []byte) ([]byte, error) {
	settings := Settings{}

	err := json.Unmarshal(payload, &settings)
	if err != nil {
		// this happens when one the payload cannot be unmarshaled
		// or one of the user-defined regular expressions is invalid
		return kubewarden.RejectSettings(
			kubewarden.Message(fmt.Sprintf("Provided settings are not valid: %v", err)))
	}

	valid, err := settings.Valid()
	if valid {
		return kubewarden.AcceptSettings()
	}

	return kubewarden.RejectSettings(
		kubewarden.Message(fmt.Sprintf("Provided settings are not valid: %v", err)))
}
