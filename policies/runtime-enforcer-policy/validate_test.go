package main

import (
	"testing"

	kubewarden_protocol "github.com/kubewarden/policy-sdk-go/protocol"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func Test_extractPodLabelsFromObject(t *testing.T) {
	tests := []struct {
		name string // description of this test case
		// Named input parameters for target function.
		object  kubewarden_protocol.ValidationRequest
		want    map[string]string
		wantErr bool
	}{
		{
			name: "Deployment object",
			object: kubewarden_protocol.ValidationRequest{
				Request: kubewarden_protocol.KubernetesAdmissionRequest{
					Kind: kubewarden_protocol.GroupVersionKind{
						Kind: "Deployment",
					},
					Object: []byte(`{"spec":{"template":{"metadata":{"labels":{"app":"my-app"}}}}}`),
				},
			},
			want: map[string]string{"app": "my-app"},
		},
		{
			name: "ReplicaSet object",
			object: kubewarden_protocol.ValidationRequest{
				Request: kubewarden_protocol.KubernetesAdmissionRequest{
					Kind: kubewarden_protocol.GroupVersionKind{
						Kind: "ReplicaSet",
					},
					Object: []byte(`{"spec":{"template":{"metadata":{"labels":{"app":"my-app"}}}}}`),
				},
			},
			want: map[string]string{"app": "my-app"},
		},
		{
			name: "StatefulSet object",
			object: kubewarden_protocol.ValidationRequest{
				Request: kubewarden_protocol.KubernetesAdmissionRequest{
					Kind: kubewarden_protocol.GroupVersionKind{
						Kind: "StatefulSet",
					},
					Object: []byte(`{"spec":{"template":{"metadata":{"labels":{"app":"my-app"}}}}}`),
				},
			},
			want: map[string]string{"app": "my-app"},
		},
		{
			name: "DaemonSet object",
			object: kubewarden_protocol.ValidationRequest{
				Request: kubewarden_protocol.KubernetesAdmissionRequest{
					Kind: kubewarden_protocol.GroupVersionKind{
						Kind: "DaemonSet",
					},
					Object: []byte(`{"spec":{"template":{"metadata":{"labels":{"app":"my-app"}}}}}`),
				},
			},
			want: map[string]string{"app": "my-app"},
		},
		{
			name: "CronJob object",
			object: kubewarden_protocol.ValidationRequest{
				Request: kubewarden_protocol.KubernetesAdmissionRequest{
					Kind: kubewarden_protocol.GroupVersionKind{
						Kind: "CronJob",
					},
					Object: []byte(
						`{"spec":{"jobTemplate":{"spec":{"template":{"metadata":{"labels":{"app":"my-app"}}}}}}}`,
					),
				},
			},
			want: map[string]string{"app": "my-app"},
		},
		{
			name: "Job object",
			object: kubewarden_protocol.ValidationRequest{
				Request: kubewarden_protocol.KubernetesAdmissionRequest{
					Kind: kubewarden_protocol.GroupVersionKind{
						Kind: "Job",
					},
					Object: []byte(`{"spec":{"template":{"metadata":{"labels":{"app":"my-app"}}}}}`),
				},
			},
			want: map[string]string{"app": "my-app"},
		},
		{
			name: "Pod object",
			object: kubewarden_protocol.ValidationRequest{
				Request: kubewarden_protocol.KubernetesAdmissionRequest{
					Kind: kubewarden_protocol.GroupVersionKind{
						Kind: "Pod",
					},
					Object: []byte(`{"metadata":{"labels":{"app":"my-app"}}}`),
				},
			},
			want: map[string]string{"app": "my-app"},
		},
		{
			name: "Unsupported object",
			object: kubewarden_protocol.ValidationRequest{
				Request: kubewarden_protocol.KubernetesAdmissionRequest{
					Kind: kubewarden_protocol.GroupVersionKind{
						Kind: "Service",
					},
					Object: []byte(`{"metadata":{"labels":{"app":"my-app"}}}`),
				},
			},
			wantErr: true,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, gotErr := extractPodLabelsFromObject(tt.object)
			if tt.wantErr {
				require.Error(t, gotErr, "extractPodLabelsFromObject() does not fail as expected")
			} else {
				require.NoError(t, gotErr, "extractPodLabelsFromObject() failed unexpectedly")
			}
			assert.Equalf(t, tt.want, got, "extractPodLabelsFromObject() = got %v, want %v", got, tt.want)
		})
	}
}
