package main

import (
	"encoding/json"
	"fmt"
	"net/http"
	"strings"

	appsv1 "github.com/kubewarden/k8s-objects/api/apps/v1"
	batchv1 "github.com/kubewarden/k8s-objects/api/batch/v1"
	corev1 "github.com/kubewarden/k8s-objects/api/core/v1"
	kubewarden "github.com/kubewarden/policy-sdk-go"
	"github.com/kubewarden/policy-sdk-go/pkg/capabilities"
	"github.com/kubewarden/policy-sdk-go/pkg/capabilities/kubernetes"
	kubewarden_protocol "github.com/kubewarden/policy-sdk-go/protocol"
)

const (
	// PolicyLabelKey is defined at:
	// https://github.com/rancher-sandbox/runtime-enforcer/blob/242bc76dcf0593110ee0a8edb2aaf2eb5b726fe8/api/v1alpha1/workloadpolicyproposal_types.go#L16
	PolicyLabelKey = "security.rancher.io/policy"
)

var (
	host = capabilities.NewHost()
)

func extractPodLabelsFromObject(object kubewarden_protocol.ValidationRequest) (map[string]string, error) {
	switch object.Request.Kind.Kind {
	case "Deployment":
		deployment := appsv1.Deployment{}
		if err := json.Unmarshal(object.Request.Object, &deployment); err != nil {
			return nil, err
		}
		return deployment.Spec.Template.Metadata.Labels, nil
	case "ReplicaSet":
		replicaset := appsv1.ReplicaSet{}
		if err := json.Unmarshal(object.Request.Object, &replicaset); err != nil {
			return nil, err
		}
		return replicaset.Spec.Template.Metadata.Labels, nil
	case "StatefulSet":
		statefulset := appsv1.StatefulSet{}
		if err := json.Unmarshal(object.Request.Object, &statefulset); err != nil {
			return nil, err
		}
		return statefulset.Spec.Template.Metadata.Labels, nil
	case "DaemonSet":
		daemonset := appsv1.DaemonSet{}
		if err := json.Unmarshal(object.Request.Object, &daemonset); err != nil {
			return nil, err
		}
		return daemonset.Spec.Template.Metadata.Labels, nil
	case "CronJob":
		cronjob := batchv1.CronJob{}
		if err := json.Unmarshal(object.Request.Object, &cronjob); err != nil {
			return nil, err
		}
		return cronjob.Spec.JobTemplate.Spec.Template.Metadata.Labels, nil
	case "Job":
		job := batchv1.Job{}
		if err := json.Unmarshal(object.Request.Object, &job); err != nil {
			return nil, err
		}
		return job.Spec.Template.Metadata.Labels, nil
	case "Pod":
		pod := corev1.Pod{}
		if err := json.Unmarshal(object.Request.Object, &pod); err != nil {
			return nil, err
		}
		return pod.Metadata.Labels, nil
	default:
		return nil, fmt.Errorf("non-supported type received: %s", object.Request.Kind.Kind)
	}
}

func validate(payload []byte) ([]byte, error) {
	// Create a ValidationRequest instance from the incoming payload
	validationRequest := kubewarden_protocol.ValidationRequest{}
	err := json.Unmarshal(payload, &validationRequest)
	if err != nil {
		return kubewarden.RejectRequest(
			kubewarden.Message(err.Error()),
			kubewarden.Code(http.StatusBadRequest))
	}

	podLabels, err := extractPodLabelsFromObject(validationRequest)
	if err != nil {
		return kubewarden.RejectRequest(kubewarden.Message(err.Error()), kubewarden.Code(http.StatusBadRequest))
	}

	wpName, wpReferenced := podLabels[PolicyLabelKey]
	if !wpReferenced {
		return kubewarden.AcceptRequest()
	}

	// We only verifies if the WorkloadPolicy exists, we don't care about its content.
	_, err = kubernetes.GetResource(&host, kubernetes.GetResourceRequest{
		APIVersion: "security.rancher.io/v1alpha1",
		Kind:       "WorkloadPolicy",
		Name:       wpName,
		Namespace:  &validationRequest.Request.Namespace,
	})

	if err != nil {
		if strings.Contains(err.Error(), "Cannot find security.rancher.io/v1alpha1/WorkloadPolicy") {
			return kubewarden.RejectRequest(
				kubewarden.Message(
					fmt.Sprintf(
						"The WorkloadPolicy '%s/%s' specified in the %s '%s/%s' is not found",
						validationRequest.Request.Namespace,
						wpName,
						validationRequest.Request.Kind.Kind,
						validationRequest.Request.Namespace,
						validationRequest.Request.Name,
					)),
				kubewarden.Code(http.StatusForbidden),
			)
		}
		return kubewarden.RejectRequest(
			kubewarden.Message(
				fmt.Sprintf(
					"Failed to read the WorkloadPolicy '%s/%s' specified in the %s '%s/%s': %v",
					validationRequest.Request.Namespace,
					wpName,
					validationRequest.Request.Kind.Kind,
					validationRequest.Request.Namespace,
					validationRequest.Request.Name,
					err,
				)),
			kubewarden.Code(http.StatusInternalServerError),
		)
	}

	return kubewarden.AcceptRequest()
}
