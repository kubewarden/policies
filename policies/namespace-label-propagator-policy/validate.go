package main

import (
	"encoding/json"
	"fmt"
	"net/http"
	"strings"

	appsv1 "github.com/kubewarden/k8s-objects/api/apps/v1"
	batchv1 "github.com/kubewarden/k8s-objects/api/batch/v1"
	corev1 "github.com/kubewarden/k8s-objects/api/core/v1"
	metav1 "github.com/kubewarden/k8s-objects/apimachinery/pkg/apis/meta/v1"
	kubewarden "github.com/kubewarden/policy-sdk-go"
	"github.com/kubewarden/policy-sdk-go/pkg/capabilities"
	kubernetes "github.com/kubewarden/policy-sdk-go/pkg/capabilities/kubernetes"
	kubewarden_protocol "github.com/kubewarden/policy-sdk-go/protocol"
)

const (
	DeploymentKind            = "deployment"
	ReplicasetKind            = "replicaset"
	StatefulsetKind           = "statefulset"
	DaemonsetKind             = "daemonset"
	ReplicationcontrollerKind = "replicationcontroller"
	CronjobKind               = "cronjob"
	JobKind                   = "job"
	PodKind                   = "pod"
)

var host = capabilities.NewHost()

func getNamespace(validationRequest kubewarden_protocol.ValidationRequest) (*corev1.Namespace, error) {
	if len(validationRequest.Request.Namespace) == 0 {
		return nil, fmt.Errorf("admission request is missing namespace")
	}

	resourceRequest := kubernetes.GetResourceRequest{
		APIVersion: "v1",
		Kind:       "Namespace",
		Name:       validationRequest.Request.Namespace,
	}

	responseBytes, err := kubernetes.GetResource(&host, resourceRequest)
	if err != nil {
		return nil, fmt.Errorf("cannot get namespace data: %w", err)
	}
	namespace := &corev1.Namespace{}
	if err = json.Unmarshal(responseBytes, namespace); err != nil {
		return nil, fmt.Errorf("cannot parse namespace data: %w", err)
	}
	return namespace, nil
}

func validateResourceLabels(
	namespaceLabels map[string]string,
	request kubewarden_protocol.ValidationRequest,
	settings Settings,
) ([]byte, error) {
	labelsToPropagate := make(map[string]string)
	for _, label := range settings.PropagatedLabels {
		if value, namespaceHasLabel := namespaceLabels[label]; namespaceHasLabel {
			labelsToPropagate[label] = value
		}
	}
	return updateResourceLabels(request, labelsToPropagate)
}

// propagateLabels ensures the labels defined in the meta object contains the
// same labels defined in the `labelsToPropagate` map. Returns `true` when
// the meta object has been changed.
func propagateLabels(meta *metav1.ObjectMeta, labelsToPropagate map[string]string) bool {
	if meta.Labels == nil {
		meta.Labels = make(map[string]string)
	}

	hasMutation := false
	for label, newValue := range labelsToPropagate {
		if oldValue, hasLabel := meta.Labels[label]; !hasLabel || oldValue != newValue {
			meta.Labels[label] = newValue
			hasMutation = true
		}
	}
	return hasMutation
}

// applyLabelPropagation propagates labelsToPropagate to objMeta and, when
// podTemplateMeta is not nil, to the pod template metadata too. It returns
// the mutation response for obj if anything changed, or an accept response
// otherwise.
func applyLabelPropagation(
	obj any,
	objMeta *metav1.ObjectMeta,
	podTemplateMeta *metav1.ObjectMeta,
	labelsToPropagate map[string]string,
) ([]byte, error) {
	objChanged := propagateLabels(objMeta, labelsToPropagate)

	podSpecChanged := false
	if podTemplateMeta != nil {
		podSpecChanged = propagateLabels(podTemplateMeta, labelsToPropagate)
	}

	if objChanged || podSpecChanged {
		return kubewarden.MutateRequest(obj)
	}

	return kubewarden.AcceptRequest()
}

func updateResourceLabels(
	object kubewarden_protocol.ValidationRequest,
	labelsToPropagate map[string]string,
) ([]byte, error) {
	switch strings.ToLower(object.Request.Kind.Kind) {
	case DeploymentKind:
		deployment := appsv1.Deployment{}
		if err := json.Unmarshal(object.Request.Object, &deployment); err != nil {
			return nil, err
		}
		return applyLabelPropagation(
			deployment,
			deployment.Metadata,
			deployment.Spec.Template.Metadata,
			labelsToPropagate,
		)
	case ReplicasetKind:
		replicaset := appsv1.ReplicaSet{}
		if err := json.Unmarshal(object.Request.Object, &replicaset); err != nil {
			return nil, err
		}
		return applyLabelPropagation(
			replicaset,
			replicaset.Metadata,
			replicaset.Spec.Template.Metadata,
			labelsToPropagate,
		)
	case StatefulsetKind:
		statefulset := appsv1.StatefulSet{}
		if err := json.Unmarshal(object.Request.Object, &statefulset); err != nil {
			return nil, err
		}
		return applyLabelPropagation(
			statefulset,
			statefulset.Metadata,
			statefulset.Spec.Template.Metadata,
			labelsToPropagate,
		)
	case DaemonsetKind:
		daemonset := appsv1.DaemonSet{}
		if err := json.Unmarshal(object.Request.Object, &daemonset); err != nil {
			return nil, err
		}
		return applyLabelPropagation(daemonset, daemonset.Metadata, daemonset.Spec.Template.Metadata, labelsToPropagate)
	case ReplicationcontrollerKind:
		replicationController := corev1.ReplicationController{}
		if err := json.Unmarshal(object.Request.Object, &replicationController); err != nil {
			return nil, err
		}
		return applyLabelPropagation(
			replicationController,
			replicationController.Metadata,
			replicationController.Spec.Template.Metadata,
			labelsToPropagate,
		)
	case CronjobKind:
		cronjob := batchv1.CronJob{}
		if err := json.Unmarshal(object.Request.Object, &cronjob); err != nil {
			return nil, err
		}
		return applyLabelPropagation(
			cronjob,
			cronjob.Metadata,
			cronjob.Spec.JobTemplate.Spec.Template.Metadata,
			labelsToPropagate,
		)
	case JobKind:
		job := batchv1.Job{}
		if err := json.Unmarshal(object.Request.Object, &job); err != nil {
			return nil, err
		}
		return applyLabelPropagation(job, job.Metadata, job.Spec.Template.Metadata, labelsToPropagate)
	case PodKind:
		pod := corev1.Pod{}
		if err := json.Unmarshal(object.Request.Object, &pod); err != nil {
			return nil, err
		}
		return applyLabelPropagation(pod, pod.Metadata, nil, labelsToPropagate)
	default:
		return nil, fmt.Errorf(
			"object should be one of these kinds: Deployment, ReplicaSet, StatefulSet, DaemonSet, ReplicationController, Job, CronJob, Pod. Found %s",
			object.Request.Kind.Kind,
		)
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

	// Create a Settings instance from the ValidationRequest object
	settings, err := NewSettingsFromValidationReq(&validationRequest)
	if err != nil {
		return kubewarden.RejectRequest(
			kubewarden.Message(err.Error()),
			kubewarden.Code(http.StatusBadRequest))
	}

	namespace, err := getNamespace(validationRequest)
	if err != nil {
		return kubewarden.RejectRequest(kubewarden.Message(err.Error()), kubewarden.Code(http.StatusBadRequest))
	}

	return validateResourceLabels(namespace.Metadata.Labels, validationRequest, settings)
}
