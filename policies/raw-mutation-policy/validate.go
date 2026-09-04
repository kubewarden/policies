package main

import (
	"encoding/json"
	"net/http"
	"strings"

	mapset "github.com/deckarep/golang-set/v2"
	onelog "github.com/francoispqt/onelog"
	kubewarden "github.com/kubewarden/policy-sdk-go"
)

func validate(payload []byte) ([]byte, error) {
	validationRequest := RawValidationRequest{}
	validationRequest.Settings = Settings{
		ForbiddenResources: mapset.NewSet[string](),
	}

	decoder := json.NewDecoder(strings.NewReader(string(payload)))
	decoder.DisallowUnknownFields()
	err := decoder.Decode(&validationRequest)
	if err != nil {
		return kubewarden.RejectRequest(
			kubewarden.Message(err.Error()),
			kubewarden.Code(http.StatusBadRequest))
	}

	request := validationRequest.Request
	settings := validationRequest.Settings

	logger.DebugWithFields("validating request", func(e onelog.Entry) {
		e.String("user", request.User)
		e.String("action", request.Action)
		e.String("resource", request.Resource)
	})

	if settings.ForbiddenResources.Contains(request.Resource) {
		request.Resource = settings.DefaultResource
		return kubewarden.MutateRequest(request)
	}

	return kubewarden.AcceptRequest()
}
