package main

import (
	"encoding/json"
	"fmt"
	"log"

	kyvernov1 "github.com/kyverno/kyverno/api/kyverno/v1"
	"k8s.io/apimachinery/pkg/util/yaml"
)

func validateSettings(input []byte) []byte {
	var response SettingsValidationResponse

	settings := &CliPolicySettings{}
	if err := json.Unmarshal(input, settings); err != nil {
		response = RejectSettings(Message(fmt.Sprintf("cannot unmarshal settings: %v", err)))
	} else {
		response = validateCliSettings(settings)
	}

	responseBytes, err := json.Marshal(&response)
	if err != nil {
		log.Fatalf("cannot marshal validation response: %v", err)
	}
	return responseBytes
}

func validateCliSettings(settings *CliPolicySettings) SettingsValidationResponse {
	// build policy
	policyBytes, err := yaml.ToJSON([]byte(settings.Policy))
	if err != nil {
		return RejectSettings(Message(fmt.Sprintf("failed to convert policy to JSON: %v", err)))
	}

	policy := &kyvernov1.ClusterPolicy{}
	err = json.Unmarshal(policyBytes, &policy)
	if err != nil {
		return RejectSettings(Message(fmt.Sprintf("cannot unmarshal policy: %v", err)))
	}
	if policy == nil {
		return RejectSettings(Message("policy cannot be empty"))
	}

	//TODO: perform more validations

	return AcceptSettings()
}
