package main

import (
	"encoding/json"
	"fmt"

	"testing"

	appsv1 "github.com/kubewarden/k8s-objects/api/apps/v1"
	batchv1 "github.com/kubewarden/k8s-objects/api/batch/v1"
	corev1 "github.com/kubewarden/k8s-objects/api/core/v1"
	metav1 "github.com/kubewarden/k8s-objects/apimachinery/pkg/apis/meta/v1"

	"github.com/kubewarden/policy-sdk-go/pkg/capabilities/kubernetes"
	"github.com/kubewarden/policy-sdk-go/pkg/capabilities/mocks"
	kubewarden_protocol "github.com/kubewarden/policy-sdk-go/protocol"
	kubewarden_testing "github.com/kubewarden/policy-sdk-go/testing"
)

const shouldAccept = true
const expectMutation = true
const noMutation = false
const testNamespace = "default"

func buildValidationRequest(propagatedLabels []string, resource any, kind string) ([]byte, error) {
	settings := Settings{PropagatedLabels: propagatedLabels}
	payload, err := kubewarden_testing.BuildValidationRequest(resource, &settings)

	if err != nil {
		return nil, err
	}
	payload, err = updateValidationRequestKindAndNamespace(payload, kind)
	if err != nil {
		return nil, err
	}
	return payload, nil
}

func basicResposeValidation(
	responsePayload []byte,
	accepted, shouldMutate bool,
) (*kubewarden_protocol.ValidationResponse, error) {
	var response kubewarden_protocol.ValidationResponse
	if err := json.Unmarshal(responsePayload, &response); err != nil {
		return nil, fmt.Errorf("Unexpected error: %+w", err)
	}

	if response.Accepted != accepted {
		return nil, fmt.Errorf("Unexpected rejection: msg %s - code %d", *response.Message, *response.Code)
	}

	if response.MutatedObject == nil && shouldMutate {
		return nil, fmt.Errorf("Missing mutated resource")
	}
	return &response, nil
}

func validateLabels(resourceLabels, expectedLabels map[string]string) error {
	for expectedLabel, expectedValue := range expectedLabels {
		if resourceValue, found := resourceLabels[expectedLabel]; found {
			if resourceValue != expectedValue {
				return fmt.Errorf(
					"Resource label \"%s\" expected value:  \"%s\". Found \"%s\"",
					expectedLabel,
					expectedValue,
					resourceValue,
				)
			}
		} else {
			return fmt.Errorf("Mutated resource missing label \"%s\"", expectedLabel)
		}
	}

	if len(resourceLabels) != len(expectedLabels) {
		return fmt.Errorf(
			"Mutated resource contains %d labels. But the expected is %d",
			len(resourceLabels),
			len(expectedLabels),
		)
	}
	return nil
}

func updateValidationRequestKindAndNamespace(payload []byte, kind string) ([]byte, error) {
	validationRequest := kubewarden_protocol.ValidationRequest{}
	err := json.Unmarshal(payload, &validationRequest)
	if err != nil {
		return nil, err
	}
	validationRequest.Request.Kind.Kind = kind
	validationRequest.Request.Namespace = testNamespace
	return json.Marshal(validationRequest)
}

func TestPodWithNoLabels(t *testing.T) {
	propagatedLabels := []string{"testing"}
	namespaceLabels := map[string]string{
		"testing":  "foo",
		"testing2": "zpto",
	}
	expectedLabels := map[string]string{
		"testing": "foo",
	}

	resource := corev1.Pod{
		Metadata: &metav1.ObjectMeta{
			Name:      "test",
			Namespace: "default",
		},
	}

	payload, err := buildValidationRequest(propagatedLabels, resource, PodKind)
	if err != nil {
		t.Errorf("Unexpected error: %+v", err)
	}

	wapcRequest, err := json.Marshal(&kubernetes.GetResourceRequest{
		APIVersion:   "v1",
		Kind:         "Namespace",
		Name:         "default",
		DisableCache: false,
	})
	if err != nil {
		t.Errorf("Cannot create wapcRequest payload: %+v", err)
	}

	wapcResponse, err := json.Marshal(&corev1.Namespace{
		Metadata: &metav1.ObjectMeta{
			Labels: namespaceLabels,
		},
	})
	if err != nil {
		t.Errorf("Cannot create wapcResponse payload: %+v", err)
	}

	wapcClient := mocks.NewMockWapcClient(t)
	wapcClient.On("HostCall", "kubewarden", "kubernetes", "get_resource",
		wapcRequest).Return(wapcResponse, nil)

	host.Client = wapcClient

	responsePayload, err := validate(payload)
	if err != nil {
		t.Errorf("Unexpected error: %+v", err)
	}

	response, err := basicResposeValidation(responsePayload, shouldAccept, expectMutation)
	if err != nil {
		t.Errorf("Unexpected error: %+v", err)
	}

	mutatedResourceJSON, err := json.Marshal(response.MutatedObject.(map[string]any))
	if err != nil {
		t.Errorf("Unexpected error: %+v", err)
	}

	if err := json.Unmarshal(mutatedResourceJSON, &resource); err != nil {
		t.Errorf("Unexpected error: %+v", err)
	}

	if err := validateLabels(resource.Metadata.Labels, expectedLabels); err != nil {
		t.Error(err.Error())
	}
}

func TestPodLabelsShouldNotMutateWithItHasTheExpectedValue(t *testing.T) {
	propagatedLabels := []string{"testing"}
	namespaceLabels := map[string]string{
		"testing":  "foo",
		"testing2": "zpto",
	}

	resource := corev1.Pod{
		Metadata: &metav1.ObjectMeta{
			Name:      "test",
			Namespace: "default",
			Labels: map[string]string{
				"testing":  "foo",
				"testing2": "zzz",
			},
		},
	}

	payload, err := buildValidationRequest(propagatedLabels, resource, PodKind)
	if err != nil {
		t.Errorf("Unexpected error: %+v", err)
	}

	wapcRequest, err := json.Marshal(&kubernetes.GetResourceRequest{
		APIVersion:   "v1",
		Kind:         "Namespace",
		Name:         "default",
		DisableCache: false,
	})
	if err != nil {
		t.Errorf("Cannot create wapcRequest payload: %+v", err)
	}

	wapcResponse, err := json.Marshal(&corev1.Namespace{
		Metadata: &metav1.ObjectMeta{
			Labels: namespaceLabels,
		},
	})
	if err != nil {
		t.Errorf("Cannot create wapcResponse payload: %+v", err)
	}

	wapcClient := mocks.NewMockWapcClient(t)
	wapcClient.On("HostCall", "kubewarden", "kubernetes", "get_resource",
		wapcRequest).Return(wapcResponse, nil)

	host.Client = wapcClient

	responsePayload, err := validate(payload)
	if err != nil {
		t.Errorf("Unexpected error: %+v", err)
	}

	_, err = basicResposeValidation(responsePayload, shouldAccept, noMutation)
	if err != nil {
		t.Errorf("Unexpected error: %+v", err)
	}
}

// overwriteLabelsTestCase describes a single
// TestLabelsShouldOverwrittenLabelsOnlyDefinedInSettings test case.
type overwriteLabelsTestCase struct {
	propagatedLabels []string
	namespaceLabels  map[string]string
	expectedLabels   map[string]string
	resource         any
	kind             string
	accept           bool
	mutate           bool
}

func TestLabelsShouldOverwrittenLabelsOnlyDefinedInSettings(t *testing.T) {
	cases := []overwriteLabelsTestCase{
		{
			[]string{"testing"},
			map[string]string{"testing": "foo", "testing2": "zpto"},
			map[string]string{"testing": "foo"},
			corev1.Pod{Metadata: &metav1.ObjectMeta{Name: "test", Namespace: "default"}},
			PodKind,
			shouldAccept, expectMutation,
		},

		{
			[]string{"testing"},
			map[string]string{"testing": "foo", "testing2": "zpto"},
			map[string]string{"testing": "foo", "testing2": "zzz"},
			batchv1.Job{
				Metadata: &metav1.ObjectMeta{
					Name:      "test",
					Namespace: "default",
					Labels:    map[string]string{"testing": "bar", "testing2": "zzz"},
				},
				Spec: &batchv1.JobSpec{
					Template: &corev1.PodTemplateSpec{
						Metadata: &metav1.ObjectMeta{
							Name:   "podtest",
							Labels: map[string]string{"testing": "pod-bar", "testing2": "zzz"},
						},
					},
				},
			},
			JobKind,
			shouldAccept, expectMutation,
		},
		{
			[]string{"testing"},
			map[string]string{"testing": "foo", "testing2": "zpto"},
			map[string]string{"testing": "foo", "testing2": "zzz"},
			batchv1.CronJob{
				Metadata: &metav1.ObjectMeta{
					Name:      "test",
					Namespace: "default",
					Labels:    map[string]string{"testing": "bar", "testing2": "zzz"},
				},
				Spec: &batchv1.CronJobSpec{
					JobTemplate: &batchv1.JobTemplateSpec{
						Spec: &batchv1.JobSpec{
							Template: &corev1.PodTemplateSpec{
								Metadata: &metav1.ObjectMeta{
									Name:   "podtest",
									Labels: map[string]string{"testing": "pod-bar", "testing2": "zzz"},
								},
							},
						},
					},
				},
			},
			CronjobKind,
			shouldAccept, expectMutation,
		},

		{
			[]string{"testing"},
			map[string]string{"testing": "foo", "testing2": "zpto"},
			map[string]string{"testing": "foo", "testing2": "zzz"},
			corev1.ReplicationController{
				Metadata: &metav1.ObjectMeta{
					Name:      "test",
					Namespace: "default",
					Labels:    map[string]string{"testing": "bar", "testing2": "zzz"},
				},
				Spec: &corev1.ReplicationControllerSpec{
					Template: &corev1.PodTemplateSpec{
						Metadata: &metav1.ObjectMeta{
							Name:   "podtest",
							Labels: map[string]string{"testing": "pod-bar", "testing2": "zzz"},
						},
					},
				},
			},
			ReplicationcontrollerKind,
			shouldAccept, expectMutation,
		},
		{
			[]string{"testing"},
			map[string]string{"testing": "foo", "testing2": "zpto"},
			map[string]string{"testing": "foo", "testing2": "zzz"},
			appsv1.DaemonSet{
				Metadata: &metav1.ObjectMeta{
					Name:      "test",
					Namespace: "default",
					Labels:    map[string]string{"testing": "bar", "testing2": "zzz"},
				},
				Spec: &appsv1.DaemonSetSpec{
					Template: &corev1.PodTemplateSpec{
						Metadata: &metav1.ObjectMeta{
							Name:   "podtest",
							Labels: map[string]string{"testing": "pod-bar", "testing2": "zzz"},
						},
					},
				},
			},
			DaemonsetKind,
			shouldAccept, expectMutation,
		},
		{
			[]string{"testing"},
			map[string]string{"testing": "foo", "testing2": "zpto"},
			map[string]string{"testing": "foo", "testing2": "zzz"},
			appsv1.StatefulSet{
				Metadata: &metav1.ObjectMeta{
					Name:      "test",
					Namespace: "default",
					Labels:    map[string]string{"testing": "bar", "testing2": "zzz"},
				},
				Spec: &appsv1.StatefulSetSpec{
					Template: &corev1.PodTemplateSpec{
						Metadata: &metav1.ObjectMeta{
							Name:   "podtest",
							Labels: map[string]string{"testing": "pod-bar", "testing2": "zzz"},
						},
					},
				},
			},
			StatefulsetKind,
			shouldAccept, expectMutation,
		},
		{

			[]string{"testing"},
			map[string]string{"testing": "foo", "testing2": "zpto"},
			map[string]string{"testing": "foo", "testing2": "zzz"},
			appsv1.ReplicaSet{
				Metadata: &metav1.ObjectMeta{
					Name:      "test",
					Namespace: "default",
					Labels:    map[string]string{"testing": "bar", "testing2": "zzz"},
				},
				Spec: &appsv1.ReplicaSetSpec{
					Template: &corev1.PodTemplateSpec{
						Metadata: &metav1.ObjectMeta{
							Name:   "podtest",
							Labels: map[string]string{"testing": "pod-bar", "testing2": "zzz"},
						},
					},
				},
			},
			ReplicasetKind,
			shouldAccept, expectMutation,
		},
		{
			[]string{"testing"},
			map[string]string{"testing": "foo", "testing2": "zpto"},
			map[string]string{"testing": "foo", "testing2": "zzz"},
			appsv1.Deployment{
				Metadata: &metav1.ObjectMeta{
					Name:      "test",
					Namespace: "default",
					Labels:    map[string]string{"testing": "bar", "testing2": "zzz"},
				},
				Spec: &appsv1.DeploymentSpec{
					Template: &corev1.PodTemplateSpec{
						Metadata: &metav1.ObjectMeta{
							Name:   "podtest",
							Labels: map[string]string{"testing": "pod-bar", "testing2": "zzz"},
						},
					},
				},
			},
			DeploymentKind,
			shouldAccept, expectMutation,
		},
	}

	for _, tc := range cases {
		t.Run(tc.kind, func(t *testing.T) {
			runOverwriteLabelsTestCase(t, tc)
		})
	}
}

// runOverwriteLabelsTestCase executes a single
// TestLabelsShouldOverwrittenLabelsOnlyDefinedInSettings test case.
func runOverwriteLabelsTestCase(t *testing.T, tc overwriteLabelsTestCase) {
	payload, err := buildValidationRequest(tc.propagatedLabels, tc.resource, tc.kind)
	if err != nil {
		t.Errorf("Unexpected error: %+v", err)
	}

	wapcRequest, err := json.Marshal(&kubernetes.GetResourceRequest{
		APIVersion:   "v1",
		Kind:         "Namespace",
		Name:         "default",
		DisableCache: false,
	})
	if err != nil {
		t.Errorf("Cannot create wapcRequest payload: %+v", err)
	}

	wapcResponse, err := json.Marshal(&corev1.Namespace{
		Metadata: &metav1.ObjectMeta{
			Labels: tc.namespaceLabels,
		},
	})
	if err != nil {
		t.Errorf("Cannot create wapcResponse payload: %+v", err)
	}

	wapcClient := mocks.NewMockWapcClient(t)
	wapcClient.On("HostCall", "kubewarden", "kubernetes", "get_resource",
		wapcRequest).Return(wapcResponse, nil)

	host.Client = wapcClient

	responsePayload, err := validate(payload)
	if err != nil {
		t.Errorf("Unexpected error: %+v", err)
	}

	response, err := basicResposeValidation(responsePayload, tc.accept, tc.mutate)
	if err != nil {
		t.Errorf("Unexpected error: %+v", err)
	}

	mutatedResourceJSON, err := json.Marshal(response.MutatedObject.(map[string]any))
	if err != nil {
		t.Errorf("Unexpected error: %+v", err)
	}

	validateMutatedResourceLabels(t, tc, mutatedResourceJSON)
}

// validateMutatedResourceLabels unmarshals mutatedResourceJSON into the
// concrete type matching tc.kind, and checks that its labels (and, when
// applicable, its pod template labels) match tc.expectedLabels.
// resourceLabelMetadata is the ObjectMeta of a mutated resource and,
// when applicable, the ObjectMeta of its pod template (nil otherwise).
type resourceLabelMetadata struct {
	objMeta         *metav1.ObjectMeta
	podTemplateMeta *metav1.ObjectMeta
}

// resourceLabelExtractors maps each supported kind to a function able to
// unmarshal a mutated resource JSON payload and extract its label metadata.
var resourceLabelExtractors = map[string]func([]byte) (resourceLabelMetadata, error){
	PodKind: func(mutatedResourceJSON []byte) (resourceLabelMetadata, error) {
		resource := corev1.Pod{}
		if err := json.Unmarshal(mutatedResourceJSON, &resource); err != nil {
			return resourceLabelMetadata{}, err
		}
		return resourceLabelMetadata{objMeta: resource.Metadata}, nil
	},
	DeploymentKind: func(mutatedResourceJSON []byte) (resourceLabelMetadata, error) {
		resource := appsv1.Deployment{}
		if err := json.Unmarshal(mutatedResourceJSON, &resource); err != nil {
			return resourceLabelMetadata{}, err
		}
		return resourceLabelMetadata{objMeta: resource.Metadata, podTemplateMeta: resource.Spec.Template.Metadata}, nil
	},
	ReplicasetKind: func(mutatedResourceJSON []byte) (resourceLabelMetadata, error) {
		resource := appsv1.ReplicaSet{}
		if err := json.Unmarshal(mutatedResourceJSON, &resource); err != nil {
			return resourceLabelMetadata{}, err
		}
		return resourceLabelMetadata{objMeta: resource.Metadata, podTemplateMeta: resource.Spec.Template.Metadata}, nil
	},
	DaemonsetKind: func(mutatedResourceJSON []byte) (resourceLabelMetadata, error) {
		resource := appsv1.DaemonSet{}
		if err := json.Unmarshal(mutatedResourceJSON, &resource); err != nil {
			return resourceLabelMetadata{}, err
		}
		return resourceLabelMetadata{objMeta: resource.Metadata, podTemplateMeta: resource.Spec.Template.Metadata}, nil
	},
	StatefulsetKind: func(mutatedResourceJSON []byte) (resourceLabelMetadata, error) {
		resource := appsv1.StatefulSet{}
		if err := json.Unmarshal(mutatedResourceJSON, &resource); err != nil {
			return resourceLabelMetadata{}, err
		}
		return resourceLabelMetadata{objMeta: resource.Metadata, podTemplateMeta: resource.Spec.Template.Metadata}, nil
	},
	ReplicationcontrollerKind: func(mutatedResourceJSON []byte) (resourceLabelMetadata, error) {
		resource := corev1.ReplicationController{}
		if err := json.Unmarshal(mutatedResourceJSON, &resource); err != nil {
			return resourceLabelMetadata{}, err
		}
		return resourceLabelMetadata{objMeta: resource.Metadata, podTemplateMeta: resource.Spec.Template.Metadata}, nil
	},
	JobKind: func(mutatedResourceJSON []byte) (resourceLabelMetadata, error) {
		resource := batchv1.Job{}
		if err := json.Unmarshal(mutatedResourceJSON, &resource); err != nil {
			return resourceLabelMetadata{}, err
		}
		return resourceLabelMetadata{objMeta: resource.Metadata, podTemplateMeta: resource.Spec.Template.Metadata}, nil
	},
	CronjobKind: func(mutatedResourceJSON []byte) (resourceLabelMetadata, error) {
		resource := batchv1.CronJob{}
		if err := json.Unmarshal(mutatedResourceJSON, &resource); err != nil {
			return resourceLabelMetadata{}, err
		}
		return resourceLabelMetadata{
			objMeta:         resource.Metadata,
			podTemplateMeta: resource.Spec.JobTemplate.Spec.Template.Metadata,
		}, nil
	},
}

// validateMutatedResourceLabels unmarshals mutatedResourceJSON into the
// concrete type matching tc.kind, and checks that its labels (and, when
// applicable, its pod template labels) match tc.expectedLabels.
func validateMutatedResourceLabels(t *testing.T, tc overwriteLabelsTestCase, mutatedResourceJSON []byte) {
	extractor, ok := resourceLabelExtractors[tc.kind]
	if !ok {
		t.Errorf("Unexpected kind: %s", tc.kind)
		return
	}

	metadata, err := extractor(mutatedResourceJSON)
	if err != nil {
		t.Errorf("Unexpected error: %+v", err)
		return
	}

	if err := validateLabels(metadata.objMeta.Labels, tc.expectedLabels); err != nil {
		t.Error(err.Error())
	}
	if metadata.podTemplateMeta != nil {
		if err := validateLabels(metadata.podTemplateMeta.Labels, tc.expectedLabels); err != nil {
			t.Error(err.Error())
		}
	}
}
