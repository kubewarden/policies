package main

import (
	"context"
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"strings"

	kyvernov1 "github.com/kyverno/kyverno/api/kyverno/v1"
	"github.com/kyverno/kyverno/pkg/config"
	"github.com/kyverno/kyverno/pkg/engine"
	enginecontext "github.com/kyverno/kyverno/pkg/engine/context"
	"github.com/kyverno/kyverno/pkg/engine/factories"
	"github.com/kyverno/kyverno/pkg/engine/jmespath"
	admissionutils "github.com/kyverno/kyverno/pkg/utils/admission"
	admissionv1 "k8s.io/api/admission/v1"
	"k8s.io/apimachinery/pkg/util/yaml"
)

func validate(input []byte) []byte {
	var validationRequest ValidationRequest

	var response ValidationResponse
	err := json.Unmarshal(input, &validationRequest)
	if err != nil {
		response = RejectRequest(
			Message(fmt.Sprintf("Error deserializing validation request: %v", err)),
			Code(http.StatusBadRequest))
	} else {
		response = validateAdmissionReview(validationRequest.Settings, validationRequest.Request)
	}

	responseBytes, err := json.Marshal(&response)
	if err != nil {
		log.Fatalf("cannot marshal validation response: %v", err)
	}
	return responseBytes
}

func validateAdmissionReview(
	policySettings CliPolicySettings,
	request admissionv1.AdmissionRequest,
) ValidationResponse {
	policyBytes, err := yaml.ToJSON([]byte(policySettings.Policy))
	if err != nil {
		return RejectRequest(
			Message(fmt.Sprintf("failed to convert policy to JSON: %v", err)),
			Code(http.StatusBadRequest))
	}

	policy := &kyvernov1.ClusterPolicy{}
	err = json.Unmarshal(policyBytes, &policy)
	if err != nil {
		return RejectRequest(
			Message(fmt.Sprintf("cannot unmarshal policy: %v", err)),
			Code(http.StatusBadRequest))
	}

	cfg := config.NewDefaultConfiguration(false)
	jp := jmespath.New(cfg)

	// TODO: handle userInfo
	engineContext := enginecontext.NewContext(jp)
	if err = engineContext.AddRequest(request); err != nil {
		return RejectRequest(
			Message(fmt.Sprintf("engine context: cannot add request %v", err)),
			Code(http.StatusInternalServerError))
	}

	e := engine.NewEngine(
		cfg,
		config.NewDefaultMetricsConfiguration(),
		jp,
		nil,
		nil,
		factories.DefaultContextLoaderFactory(nil),
		nil,
	)

	newR, oldR, err := admissionutils.ExtractResources(nil, request)
	if err != nil {
		return RejectRequest(
			Message(fmt.Sprintf("cannot extract resources %v", err)),
			Code(http.StatusInternalServerError))
	}

	policyContext := engine.NewPolicyContextWithJsonContext(kyvernov1.Create, engineContext).
		WithPolicy(policy).
		WithNewResource(newR).
		WithOldResource(oldR)

	er := e.Validate(context.Background(), policyContext)

	errorMsgs := []string{}
	for index, r := range er.PolicyResponse.Rules {
		errorMsgs = append(errorMsgs, fmt.Sprintf("[%d] - %s", index, r.Message()))
	}

	if er.IsSuccessful() {
		return AcceptRequest()
	}

	return RejectRequest(Message(strings.Join(errorMsgs, ",")), NoCode)
}
