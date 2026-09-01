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
		ForbiddenResources: mapset.NewSet[string](),
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
		mutateRequest(&validationRequest.Settings, &validationRequest.Request))
}

func marshalValidationResponseOrFail(response ValidationResponse) []byte {
	responseBytes, err := json.Marshal(&response)
	if err != nil {
		log.Fatalf("cannot marshal validation response: %v", err)
	}
	return responseBytes
}

func mutateRequest(settings *Settings, request *Request) ValidationResponse {
	if settings.ForbiddenResources.Contains(request.Resource) {
		request.Resource = settings.DefaultResource
		return MutateRequest(request)
	}
	return AcceptRequest()
}
