package main

import (
	"encoding/json"
	"net/http"
	"testing"

	kubewarden_protocol "github.com/kubewarden/policy-sdk-go/protocol"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func Test_validateSettings(t *testing.T) {
	tests := []struct {
		name     string
		settings string
		valid    bool
	}{
		{name: "empty settings", settings: `{}`, valid: true},
		{name: "requireProtection true", settings: `{"requireProtection": true}`, valid: true},
		{name: "requireProtection false", settings: `{"requireProtection": false}`, valid: true},
		{name: "wrong type", settings: `{"requireProtection": "yes"}`, valid: false},
		{name: "malformed JSON", settings: `{`, valid: false},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			raw, err := validateSettings([]byte(tt.settings))
			require.NoError(t, err)

			response := kubewarden_protocol.SettingsValidationResponse{}
			require.NoError(t, json.Unmarshal(raw, &response))
			assert.Equal(t, tt.valid, response.Valid)
		})
	}
}

func Test_validate_requireProtection(t *testing.T) {
	unprotectedDeployment := `{"metadata":{"name":"my-app","namespace":"default"},"spec":{"template":{"metadata":{"labels":{"app":"my-app"}}}}}`

	tests := []struct {
		name     string
		settings string
		accepted bool
		code     uint16
	}{
		{
			name:     "null settings: unprotected workload accepted",
			settings: `null`,
			accepted: true,
		},
		{
			name:     "empty settings: unprotected workload accepted",
			settings: `{}`,
			accepted: true,
		},
		{
			name:     "requireProtection false: unprotected workload accepted",
			settings: `{"requireProtection": false}`,
			accepted: true,
		},
		{
			name:     "requireProtection true: unprotected workload rejected",
			settings: `{"requireProtection": true}`,
			accepted: false,
			code:     http.StatusForbidden,
		},
		{
			name:     "invalid settings: request rejected",
			settings: `{"requireProtection": "yes"}`,
			accepted: false,
			code:     http.StatusBadRequest,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			request := kubewarden_protocol.ValidationRequest{
				Request: kubewarden_protocol.KubernetesAdmissionRequest{
					Kind:      kubewarden_protocol.GroupVersionKind{Kind: "Deployment"},
					Name:      "my-app",
					Namespace: "default",
					Object:    []byte(unprotectedDeployment),
				},
				Settings: []byte(tt.settings),
			}
			payload, err := json.Marshal(request)
			require.NoError(t, err)

			raw, err := validate(payload)
			require.NoError(t, err)

			response := kubewarden_protocol.ValidationResponse{}
			require.NoError(t, json.Unmarshal(raw, &response))
			assert.Equal(t, tt.accepted, response.Accepted)
			if !tt.accepted {
				require.NotNil(t, response.Code)
				assert.Equal(t, tt.code, *response.Code)
				require.NotNil(t, response.Message)
				assert.NotEmpty(t, *response.Message)
			}
		})
	}
}

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
