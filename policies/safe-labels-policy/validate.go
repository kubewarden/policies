package main

import (
	"fmt"
	"net/http"
	"strings"

	mapset "github.com/deckarep/golang-set/v2"
	"github.com/kubewarden/gjson"
	kubewarden "github.com/kubewarden/policy-sdk-go"
)

func validate(payload []byte) ([]byte, error) {
	if !gjson.ValidBytes(payload) {
		return kubewarden.RejectRequest(
			kubewarden.Message("Not a valid JSON document"),
			kubewarden.Code(http.StatusBadRequest))
	}

	settings, err := NewSettingsFromValidationReq(payload)
	if err != nil {
		return kubewarden.RejectRequest(
			kubewarden.Message(err.Error()),
			kubewarden.Code(http.StatusBadRequest))
	}

	data := gjson.GetBytes(
		payload,
		"request.object.metadata.labels")

	labels := mapset.NewThreadUnsafeSet[string]()
	deniedLabelsViolations := []string{}
	constrainedLabelsViolations := []string{}

	data.ForEach(func(key, value gjson.Result) bool {
		label := key.String()
		labels.Add(label)

		if settings.DeniedLabels.Contains(label) {
			deniedLabelsViolations = append(deniedLabelsViolations, label)
			return true
		}

		regExp, found := settings.ConstrainedLabels[label]
		if found {
			// This is a constrained label
			if !regExp.Match([]byte(value.String())) {
				constrainedLabelsViolations = append(constrainedLabelsViolations, label)
				return true
			}
		}

		return true
	})

	errorMsgs := []string{}

	if len(deniedLabelsViolations) > 0 {
		errorMsgs = append(
			errorMsgs,
			fmt.Sprintf(
				"The following labels are denied: %s",
				strings.Join(deniedLabelsViolations, ","),
			))
	}

	if len(constrainedLabelsViolations) > 0 {
		errorMsgs = append(
			errorMsgs,
			fmt.Sprintf(
				"The following labels are violating user constraints: %s",
				strings.Join(constrainedLabelsViolations, ","),
			))
	}

	mandatoryLabelsViolations := settings.MandatoryLabels.Difference(labels)
	if mandatoryLabelsViolations.Cardinality() > 0 {
		violations := mandatoryLabelsViolations.ToSlice()

		errorMsgs = append(
			errorMsgs,
			fmt.Sprintf(
				"The following mandatory labels are missing: %s",
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
