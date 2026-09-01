// This file contains all the data structures that need
// to serialized and deserialized to JSON.
//
// Important: limit the number of imports inside of this file. Also, don't
// try to use interface types (or anything making use of them). This isn't
// going to play out well with TinyGo at **runtime** due to its limited
// support of Go reflection.

package main

import (
	apimachinery_pkg_apis_meta_v1 "github.com/kubewarden/k8s-objects/apimachinery/pkg/apis/meta/v1"
)

// Settings is a an empty struct because this policy has no configuration.
type Settings struct {
}

// ConditionStatus is a valid condition status.
type ConditionStatus string

const (
	ConditionTrue    ConditionStatus = "True"
	ConditionFalse   ConditionStatus = "False"
	ConditionUnknown ConditionStatus = "Unknown"
)

// Project is a Rancher Custom Resource Definition
// Taken from https://github.com/rancher/types/blob/release/v2.4/apis/management.cattle.io/v3/authz_types.go
type Project struct {
	// APIVersion defines the versioned schema of this representation of an object. Servers should convert recognized schemas to the latest internal value, and may reject unrecognized values. More info: https://git.k8s.io/community/contributors/devel/sig-architecture/api-conventions.md#resources
	APIVersion string `json:"apiVersion,omitempty"`

	// Kind is a string value representing the REST resource this object represents. Servers may infer this from the endpoint the client submits requests to. Cannot be updated. In CamelCase. More info: https://git.k8s.io/community/contributors/devel/sig-architecture/api-conventions.md#types-kinds
	Kind string `json:"kind,omitempty"`

	// Standard object's metadata. More info: https://git.k8s.io/community/contributors/devel/sig-architecture/api-conventions.md#metadata
	Metadata *apimachinery_pkg_apis_meta_v1.ObjectMeta `json:"metadata,omitempty"`

	//// Specification of the desired behavior of the project. More info: https://git.k8s.io/community/contributors/devel/sig-architecture/api-conventions.md#spec-and-status
	Spec *ProjectSpec `json:"spec,omitempty"`

	//// Most recently observed status of the project. This data may not be up to date. Populated by the system. Read-only. More info: https://git.k8s.io/community/contributors/devel/sig-architecture/api-conventions.md#spec-and-status
	Status ProjectStatus `json:"status"`
}

// ProjectSpec contains the details of a Rancher Project
// Taken from https://github.com/rancher/types/blob/release/v2.4/apis/management.cattle.io/v3/authz_types.go
type ProjectSpec struct {
	DisplayName                   string                  `json:"displayName,omitempty"                   norman:"required"`
	Description                   string                  `json:"description"`
	ClusterName                   string                  `json:"clusterName,omitempty"                   norman:"required,type=reference[cluster]"`
	ResourceQuota                 *ProjectResourceQuota   `json:"resourceQuota,omitempty"`
	NamespaceDefaultResourceQuota *NamespaceResourceQuota `json:"namespaceDefaultResourceQuota,omitempty"`
	ContainerDefaultResourceLimit *ContainerResourceLimit `json:"containerDefaultResourceLimit,omitempty"`
	EnableProjectMonitoring       bool                    `json:"enableProjectMonitoring"                 norman:"default=false"`
}

// ProjectStatus contains the observed status of the project
// Taken from https://github.com/rancher/types/blob/release/v2.4/apis/management.cattle.io/v3/authz_types.go
type ProjectStatus struct {
	Conditions                    []ProjectCondition `json:"conditions"`
	PodSecurityPolicyTemplateName string             `json:"podSecurityPolicyTemplateId"`
	MonitoringStatus              *MonitoringStatus  `json:"monitoringStatus,omitempty"  norman:"nocreate,noupdate"`
}

// ProjectCondition contains the conditions of the project
// Taken from https://github.com/rancher/types/blob/release/v2.4/apis/management.cattle.io/v3/authz_types.go
type ProjectCondition struct {
	// Type of project condition.
	Type string `json:"type"`
	// Status of the condition, one of True, False, Unknown.
	Status ConditionStatus `json:"status"`
	// The last time this condition was updated.
	LastUpdateTime string `json:"lastUpdateTime,omitempty"`
	// Last time the condition transitioned from one status to another.
	LastTransitionTime string `json:"lastTransitionTime,omitempty"`
	// The reason for the condition's last transition.
	Reason string `json:"reason,omitempty"`
	// Human-readable message indicating details about last transition
	Message string `json:"message,omitempty"`
}

// ProjectResourceQuota describes the limit and used limits of a Project
// Taken from https://github.com/rancher/types/blob/release/v2.4/apis/management.cattle.io/v3/resource_quota_types.go
type ProjectResourceQuota struct {
	Limit     ResourceQuotaLimit `json:"limit,omitzero"`
	UsedLimit ResourceQuotaLimit `json:"usedLimit,omitzero"`
}

// NamespaceResourceQuota defines the quota limits applied to the namespace
// Taken from https://github.com/rancher/types/blob/release/v2.4/apis/management.cattle.io/v3/resource_quota_types.go
type NamespaceResourceQuota struct {
	Limit ResourceQuotaLimit `json:"limit,omitzero"`
}

// ResourceQuotaLimit defines the types of quotas that can be set
// Taken from https://github.com/rancher/types/blob/release/v2.4/apis/management.cattle.io/v3/resource_quota_types.go
type ResourceQuotaLimit struct {
	Pods                   string `json:"pods,omitempty"`
	Services               string `json:"services,omitempty"`
	ReplicationControllers string `json:"replicationControllers,omitempty"`
	Secrets                string `json:"secrets,omitempty"`
	ConfigMaps             string `json:"configMaps,omitempty"`
	PersistentVolumeClaims string `json:"persistentVolumeClaims,omitempty"`
	ServicesNodePorts      string `json:"servicesNodePorts,omitempty"`
	ServicesLoadBalancers  string `json:"servicesLoadBalancers,omitempty"`
	RequestsCPU            string `json:"requestsCpu,omitempty"`
	RequestsMemory         string `json:"requestsMemory,omitempty"`
	RequestsStorage        string `json:"requestsStorage,omitempty"`
	LimitsCPU              string `json:"limitsCpu,omitempty"`
	LimitsMemory           string `json:"limitsMemory,omitempty"`
}

// ContainerResourceLimit defines the types of limits that can be set
// Taken from https://github.com/rancher/types/blob/release/v2.4/apis/management.cattle.io/v3/resource_quota_types.go
type ContainerResourceLimit struct {
	RequestsCPU    string `json:"requestsCpu,omitempty"`
	RequestsMemory string `json:"requestsMemory,omitempty"`
	LimitsCPU      string `json:"limitsCpu,omitempty"`
	LimitsMemory   string `json:"limitsMemory,omitempty"`
}

// MonitoringStatus is taken from https://github.com/rancher/types/blob/release/v2.4/apis/management.cattle.io/v3/monitoring_types.go
type MonitoringStatus struct {
	GrafanaEndpoint string                `json:"grafanaEndpoint,omitempty"`
	Conditions      []MonitoringCondition `json:"conditions,omitempty"`
}

// ClusterConditionType is taken from https://github.com/rancher/types/blob/release/v2.4/apis/management.cattle.io/v3/cluster_types.go
type ClusterConditionType string

// MonitoringCondition taken from https://github.com/rancher/types/blob/release/v2.4/apis/management.cattle.io/v3/monitoring_types.go
type MonitoringCondition struct {
	// Type of cluster condition.
	Type ClusterConditionType `json:"type"`
	// Status of the condition, one of True, False, Unknown.
	Status ConditionStatus `json:"status"`
	// The last time this condition was updated.
	LastUpdateTime string `json:"lastUpdateTime,omitempty"`
	// Last time the condition transitioned from one status to another.
	LastTransitionTime string `json:"lastTransitionTime,omitempty"`
	// The reason for the condition's last transition.
	Reason string `json:"reason,omitempty"`
	// Human-readable message indicating details about last transition
	Message string `json:"message,omitempty"`
}
