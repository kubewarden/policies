package main

import (
	"encoding/json"
	"fmt"

	appsv1 "github.com/kubewarden/k8s-objects/api/apps/v1"
	corev1 "github.com/kubewarden/k8s-objects/api/core/v1"
	kubewarden "github.com/kubewarden/policy-sdk-go"
	"github.com/kubewarden/policy-sdk-go/pkg/capabilities"
	"github.com/kubewarden/policy-sdk-go/pkg/capabilities/kubernetes"
	kubewardenProtocol "github.com/kubewarden/policy-sdk-go/protocol"
)

// LookupError is a custom error that provides extra information.
type LookupError struct {
	StatusCode kubewarden.Code
	Message    kubewarden.Message
}

const (
	BadRequestError = 400
	NotFoundError   = 404
	InternalError   = 500
)

func (l *LookupError) Error() string {
	return fmt.Sprintf("status %d: err %v", l.StatusCode, l.Message)
}

func validate(input []byte) ([]byte, error) {
	validationRequest := kubewardenProtocol.ValidationRequest{}

	err := json.Unmarshal(input, &validationRequest)
	if err != nil {
		return kubewarden.RejectRequest(
			kubewarden.Message(fmt.Sprintf("Error deserializing validation request: %v", err)),
			kubewarden.Code(BadRequestError))
	}
	settings, err := NewSettingsFromValidationReq(&validationRequest)
	if err != nil {
		return kubewarden.RejectRequest(
			kubewarden.Message(fmt.Sprintf("Error serializing RawMessage: %v", err)),
			kubewarden.Code(BadRequestError))
	}

	return validateAdmissionReview(settings, validationRequest.Request)
}

func validateAdmissionReview(_ Settings, request kubewardenProtocol.KubernetesAdmissionRequest) ([]byte, error) {
	deployment := appsv1.Deployment{}
	err := json.Unmarshal(request.Object, &deployment)
	if err != nil {
		return kubewarden.RejectRequest(
			kubewarden.Message(fmt.Sprintf("Error deserializing request object into unstructured: %v", err)),
			kubewarden.Code(BadRequestError))
	}

	labels := deployment.Metadata.Labels
	if labels == nil {
		return kubewarden.AcceptRequest()
	}

	// Check if the app.kubernetes.io/component label is set to "api"
	// If not, accept the request
	if labels["app.kubernetes.io/component"] != "api" {
		return kubewarden.AcceptRequest()
	}

	// Get the customerID label value
	// If not set, reject the request
	customerID, found := labels["customer-id"]
	if !found {
		return kubewarden.RejectRequest(
			kubewarden.Message("Label customer-id is required for API deployments"),
			kubewarden.Code(BadRequestError))
	}

	host := capabilities.NewHost()

	namespaceList, err := findNamespacesByCustomerID(&host, customerID)
	if err != nil {
		return kubewarden.RejectRequest(
			kubewarden.Message(fmt.Sprintf("cannot query Namespaces: %v", err)),
			kubewarden.Code(InternalError))
	}

	if len(namespaceList.Items) == 0 {
		return kubewarden.RejectRequest(
			kubewarden.Message(fmt.Sprintf("Label customer-id (%s) must match namespace label", customerID)),
			kubewarden.Code(NotFoundError))
	}

	if len(namespaceList.Items) > 1 {
		return kubewarden.RejectRequest(
			kubewarden.Message(fmt.Sprintf("Multiple namespaces found with label 'customer-id=%s'", customerID)),
			kubewarden.Code(BadRequestError))
	}

	namespace := namespaceList.Items[0]
	if deployment.Metadata.Namespace != namespace.Metadata.Name {
		return kubewarden.RejectRequest(
			kubewarden.Message("Deployment must be created in the matching customer namespace"),
			kubewarden.Code(BadRequestError))
	}

	deploymentList, err := findDeploymentsByNamespace(&host, namespace.Metadata.Name)
	if err != nil {
		return kubewarden.RejectRequest(
			kubewarden.Message(fmt.Sprintf("cannot query Deployments: %v", err)),
			kubewarden.Code(InternalError))
	}

	// Check if the namespace has a database and a frontend component deployed
	if !componentDeployed(&deploymentList, "database") {
		return kubewarden.RejectRequest(
			kubewarden.Message("No database component found"),
			kubewarden.Code(NotFoundError))
	}
	if !componentDeployed(&deploymentList, "frontend") {
		return kubewarden.RejectRequest(
			kubewarden.Message("No frontend component found"),
			kubewarden.Code(NotFoundError))
	}

	// Check if the namespace has an authentication service deployed
	service, lookupErr := findAPIAuthService(&host, namespace.Metadata.Name)
	if lookupErr != nil {
		return kubewarden.RejectRequest(
			lookupErr.Message,
			lookupErr.StatusCode,
		)
	}

	if service.Metadata.Labels != nil && service.Metadata.Labels["app.kubernetes.io/part-of"] != "api" {
		return kubewarden.RejectRequest(
			kubewarden.Message("No API authentication service found"),
			kubewarden.Code(NotFoundError),
		)
	}
	if len(service.Metadata.Labels) == 0 {
		return kubewarden.RejectRequest(
			kubewarden.Message("API authentication service must have labels"),
			kubewarden.Code(NotFoundError),
		)
	}

	return kubewarden.AcceptRequest()
}

func componentDeployed(deploymentList *appsv1.DeploymentList, componentName string) bool {
	for _, deployment := range deploymentList.Items {
		if deployment.Metadata.Labels != nil &&
			deployment.Metadata.Labels["app.kubernetes.io/component"] == componentName {
			return true
		}
	}
	return false
}

func findAPIAuthService(host *capabilities.Host, namespace string) (corev1.Service, *LookupError) {
	kubeRequest := kubernetes.GetResourceRequest{
		APIVersion: "v1",
		Kind:       "Service",
		Name:       "api-auth-service",
		Namespace:  &namespace,
	}
	serviceRaw, err := kubernetes.GetResource(host, kubeRequest)
	if err != nil {
		return corev1.Service{}, &LookupError{
			Message:    kubewarden.Message(fmt.Sprintf("cannot query Service: %v", err)),
			StatusCode: kubewarden.Code(InternalError),
		}
	}

	if len(serviceRaw) == 0 {
		return corev1.Service{}, &LookupError{
			Message:    kubewarden.Message("No API authentication service found"),
			StatusCode: kubewarden.Code(NotFoundError),
		}
	}

	service := corev1.Service{}
	err = json.Unmarshal(serviceRaw, &service)
	if err != nil {
		return corev1.Service{},
			&LookupError{
				Message: kubewarden.Message(
					fmt.Sprintf("cannot unmarshall response into Service: %v", err)),
				StatusCode: kubewarden.Code(NotFoundError),
			}
	}

	return service, nil
}

func findNamespacesByCustomerID(host *capabilities.Host, customerID string) (corev1.NamespaceList, error) {
	labelSelector := "customer-id=" + customerID
	kubeRequest := kubernetes.ListAllResourcesRequest{
		APIVersion:    "v1",
		Kind:          "Namespace",
		LabelSelector: &labelSelector,
	}
	response, err := kubernetes.ListResources(host, kubeRequest)
	if err != nil {
		return corev1.NamespaceList{}, err
	}

	namespaceList := corev1.NamespaceList{}
	err = json.Unmarshal(response, &namespaceList)
	if err != nil {
		return corev1.NamespaceList{}, fmt.Errorf("cannot unmarshall response into NamespaceList: %w", err)
	}

	return namespaceList, nil
}

func findDeploymentsByNamespace(host *capabilities.Host, namespace string) (appsv1.DeploymentList, error) {
	kubeRequest := kubernetes.ListResourcesByNamespaceRequest{
		APIVersion: "apps/v1",
		Kind:       "Deployment",
		Namespace:  namespace,
	}
	response, err := kubernetes.ListResourcesByNamespace(host, kubeRequest)
	if err != nil {
		return appsv1.DeploymentList{}, err
	}

	deploymentList := appsv1.DeploymentList{}
	err = json.Unmarshal(response, &deploymentList)
	if err != nil {
		return appsv1.DeploymentList{}, fmt.Errorf("cannot unmarshall response into NamespaceList: %w", err)
	}

	return deploymentList, nil
}
