package main

import (
	"fmt"
	"strings"

	"github.com/kubewarden/rancher-project-quotas-namespace-validator/resource"
)

// QuantityParseError is a custom error raised when a string cannot be
// parsed to be be a resource.Quantity.
type QuantityParseError struct {
	Message string
	Err     error
}

func (e *QuantityParseError) Error() string {
	return fmt.Sprintf("%s: %v", e.Message, e.Err)
}

// NamespaceRequestExceedsAvailabilityError a custom error raised when
// a namespace requests more resources than available.
type NamespaceRequestExceedsAvailabilityError struct {
	requested string
	available string
}

func (e *NamespaceRequestExceedsAvailabilityError) Error() string {
	return fmt.Sprintf(
		"Namespace requested limit exceeds the availability of the project resource: requested %s, available %s",
		e.requested,
		e.available,
	)
}

// Compares the amount of resources requested by a namespace against the
// availability of a project.
//
// Returns an error when one of these situation occurs:
//   - The given strings cannot be converted to a Kubernetes Quantity
//   - The project is already out of resources
//   - The namespace has requested too much of a resource compared to the availability
//     of the project
func checkLimitVsAvailableQuota(nsLimit, prjLimit, prjUsed string) error {
	if nsLimit == "" {
		nsLimit = "0"
	}
	nsLimitQuantity, err := resource.ParseQuantity(nsLimit)
	if err != nil {
		return &QuantityParseError{
			Message: "Cannot convert namespace limit to quantity",
			Err:     err,
		}
	}

	if prjLimit == "" {
		prjLimit = "0"
	}
	prjLimitQuantity, err := resource.ParseQuantity(prjLimit)
	if err != nil {
		return &QuantityParseError{
			Message: "Cannot convert project limit to quantity",
			Err:     err,
		}
	}

	if prjUsed == "" {
		prjUsed = "0"
	}
	prjUsedQuantity, err := resource.ParseQuantity(prjUsed)
	if err != nil {
		return &QuantityParseError{
			Message: "Cannot convert project used quota to quantity",
			Err:     err,
		}
	}

	prjAvailableQuantity := prjLimitQuantity.DeepCopy()
	prjAvailableQuantity.Sub(prjUsedQuantity)

	if nsLimitQuantity.Cmp(prjAvailableQuantity) > 0 {
		return &NamespaceRequestExceedsAvailabilityError{
			requested: nsLimitQuantity.String(),
			available: prjAvailableQuantity.String(),
		}
	}

	return nil
}

func validateQuotas(project *Project, nsLimits *ResourceQuotaLimit) error {
	if project.Spec.ResourceQuota == nil {
		return nil
	}

	if nsLimits == nil {
		nsLimits = &ResourceQuotaLimit{}
	}

	limit := project.Spec.ResourceQuota.Limit
	used := project.Spec.ResourceQuota.UsedLimit

	quotas := []struct {
		name  string
		ns    string
		limit string
		used  string
	}{
		{"Pods", nsLimits.Pods, limit.Pods, used.Pods},
		{"Services", nsLimits.Services, limit.Services, used.Services},
		{
			"ReplicationControllers",
			nsLimits.ReplicationControllers,
			limit.ReplicationControllers,
			used.ReplicationControllers,
		},
		{"Secrets", nsLimits.Secrets, limit.Secrets, used.Secrets},
		{"ConfigMaps", nsLimits.ConfigMaps, limit.ConfigMaps, used.ConfigMaps},
		{
			"PersistentVolumeClaims",
			nsLimits.PersistentVolumeClaims,
			limit.PersistentVolumeClaims,
			used.PersistentVolumeClaims,
		},
		{"ServicesNodePorts", nsLimits.ServicesNodePorts, limit.ServicesNodePorts, used.ServicesNodePorts},
		{
			"ServicesLoadBalancers",
			nsLimits.ServicesLoadBalancers,
			limit.ServicesLoadBalancers,
			used.ServicesLoadBalancers,
		},
		{"RequestsCPU", nsLimits.RequestsCPU, limit.RequestsCPU, used.RequestsCPU},
		{"RequestsMemory", nsLimits.RequestsMemory, limit.RequestsMemory, used.RequestsMemory},
		{"RequestsStorage", nsLimits.RequestsStorage, limit.RequestsStorage, used.RequestsStorage},
		{"LimitsCPU", nsLimits.LimitsCPU, limit.LimitsCPU, used.LimitsCPU},
		{"LimitsMemory", nsLimits.LimitsMemory, limit.LimitsMemory, used.LimitsMemory},
	}

	errorMsgs := []string{}
	for _, quota := range quotas {
		if err := checkLimitVsAvailableQuota(quota.ns, quota.limit, quota.used); err != nil {
			errorMsgs = append(errorMsgs, fmt.Sprintf("%s limit: %v", quota.name, err))
		}
	}

	if len(errorMsgs) == 0 {
		return nil
	}

	return fmt.Errorf("%s", strings.Join(errorMsgs, ", "))
}
