package main

import (
	"encoding/json"
	"fmt"
	"net/http"
	"strings"

	mapset "github.com/deckarep/golang-set/v2"
	"github.com/kubewarden/gjson"
	kubewarden "github.com/kubewarden/policy-sdk-go"
	kubewarden_protocol "github.com/kubewarden/policy-sdk-go/protocol"
)

func validate(payload []byte) ([]byte, error) {
	// Create a ValidationRequest instance from the incoming payload
	validationRequest := kubewarden_protocol.ValidationRequest{}
	err := json.Unmarshal(payload, &validationRequest)
	if err != nil {
		return kubewarden.RejectRequest(
			kubewarden.Message(err.Error()),
			kubewarden.Code(http.StatusBadRequest))
	}

	settings, err := NewSettingsFromValidationReq(validationRequest)
	if err != nil {
		return kubewarden.RejectRequest(
			kubewarden.Message(err.Error()),
			kubewarden.Code(http.StatusBadRequest))
	}

	data := gjson.GetBytes(
		payload,
		"request.object.metadata.annotations")

	annotations := mapset.NewThreadUnsafeSet[string]()
	deniedAnnotationsViolations := []string{}
	constrainedAnnotationsViolations := []string{}

	data.ForEach(func(key, value gjson.Result) bool {
		annotation := key.String()
		annotations.Add(annotation)

		if settings.DeniedAnnotations.Contains(annotation) {
			deniedAnnotationsViolations = append(deniedAnnotationsViolations, annotation)
			return true
		}

		regExp, found := settings.ConstrainedAnnotations[annotation]
		if found {
			// This is a constrained annotation
			if !regExp.Match([]byte(value.String())) {
				constrainedAnnotationsViolations = append(constrainedAnnotationsViolations, annotation)
				return true
			}
		}

		return true
	})

	errorMsgs := []string{}

	if len(deniedAnnotationsViolations) > 0 {
		errorMsgs = append(
			errorMsgs,
			fmt.Sprintf(
				"The following annotations are not allowed: %s",
				strings.Join(deniedAnnotationsViolations, ","),
			))
	}

	if len(constrainedAnnotationsViolations) > 0 {
		errorMsgs = append(
			errorMsgs,
			fmt.Sprintf(
				"The following annotations are violating user constraints: %s",
				strings.Join(constrainedAnnotationsViolations, ","),
			))
	}

	mandatoryAnnotationsViolations := settings.MandatoryAnnotations.Difference(annotations)
	if mandatoryAnnotationsViolations.Cardinality() > 0 {
		violations := mandatoryAnnotationsViolations.ToSlice()

		errorMsgs = append(
			errorMsgs,
			fmt.Sprintf(
				"The following mandatory annotations are missing: %s",
				strings.Join(violations, ","),
			))
	}

	if len(errorMsgs) > 0 {
		return kubewarden.RejectRequest(
			kubewarden.Message(strings.Join(errorMsgs, ". ")),
			kubewarden.NoCode)
	}

	return kubewarden.AcceptRequest()
}
