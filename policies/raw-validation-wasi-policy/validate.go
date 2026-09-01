package main

import (
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"strings"

	mapset "github.com/deckarep/golang-set/v2"
)

func validate(input []byte) []byte {
	var validationRequest RawValidationRequest

	validationRequest.Settings = Settings{
		// this is required to make the unmarshal work
		ValidUsers:     mapset.NewSet[string](),
		ValidActions:   mapset.NewSet[string](),
		ValidResources: mapset.NewSet[string](),
	}
	decoder := json.NewDecoder(strings.NewReader(string(input)))
	decoder.DisallowUnknownFields()

	err := decoder.Decode(&validationRequest)
	if err != nil {
		return marshalValidationResponseOrFail(
			RejectRequest(
				Message(fmt.Sprintf("Error deserializing validation request: %v", err)),
				Code(http.StatusBadRequest)))
	}

	return marshalValidationResponseOrFail(
		validateRequest(validationRequest.Settings, validationRequest.Request))
}

func marshalValidationResponseOrFail(response ValidationResponse) []byte {
	responseBytes, err := json.Marshal(&response)
	if err != nil {
		log.Fatalf("cannot marshal validation response: %v", err)
	}

	return responseBytes
}

func validateRequest(settings Settings, request Request) ValidationResponse {
	if settings.ValidUsers.Contains(request.User) &&
		settings.ValidActions.Contains(request.Action) &&
		settings.ValidResources.Contains(request.Resource) {
		return AcceptRequest()
	}

	if !settings.ValidUsers.Contains(request.User) {
		return RejectRequest(
			Message(fmt.Sprintf("User '%s' is not allowed", request.User)),
			Code(http.StatusForbidden))
	}

	if !settings.ValidActions.Contains(request.Action) {
		return RejectRequest(
			Message(fmt.Sprintf("Action '%s' is not allowed", request.Action)),
			Code(http.StatusForbidden))
	}

	if !settings.ValidResources.Contains(request.Resource) {
		return RejectRequest(
			Message(fmt.Sprintf("Resource '%s' is not allowed", request.Resource)),
			Code(http.StatusForbidden))
	}

	return AcceptRequest()
}
