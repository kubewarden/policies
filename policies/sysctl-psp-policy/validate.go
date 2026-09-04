package main

import (
	"fmt"
	"net/http"
	"strings"

	mapset "github.com/deckarep/golang-set/v2"
	onelog "github.com/francoispqt/onelog"
	"github.com/kubewarden/gjson"
	kubewarden "github.com/kubewarden/policy-sdk-go"
)

// CreateSafeSysctlsSet returns a set with the known safe sysctls.
//
// A sysctl is called safe iff:
// - it is namespaced in the container or the pod
// - it is isolated, i.e. has no influence on any other pod on the same node.
//
// A (possibly not up-to-date) list of known safe sysctls can be found at:
// https://kubernetes.io/docs/concepts/security/pod-security-standards/#baseline
func CreateSafeSysctlsSet() mapset.Set[string] {
	safeSysctls := mapset.NewThreadUnsafeSet[string]()
	safeSysctls.Add("kernel.shm_rmid_forced")
	safeSysctls.Add("net.ipv4.ip_local_port_range")
	safeSysctls.Add("net.ipv4.tcp_syncookies")
	safeSysctls.Add("net.ipv4.ping_group_range")
	return safeSysctls
}

func validate(payload []byte) ([]byte, error) {
	settings, err := NewSettingsFromValidationReq(payload)
	if err != nil {
		return kubewarden.RejectRequest(
			kubewarden.Message(err.Error()),
			kubewarden.Code(http.StatusBadRequest))
	}

	logger.Info("validating request")

	data := gjson.GetBytes(
		payload,
		"request.object.spec.securityContext.sysctls")

	if !data.Exists() {
		// Pod specifies no sysctls, accepting
		return kubewarden.AcceptRequest()
	}

	logger.DebugWithFields("validating pod object", func(e onelog.Entry) {
		name := gjson.GetBytes(payload, "request.object.metadata.name").String()
		namespace := gjson.GetBytes(payload,
			"request.object.metadata.namespace").String()
		e.String("name", name)
		e.String("namespace", namespace)
	})

	knownSafeSysctls := CreateSafeSysctlsSet()

	// build set of prefixes from patterns of forbidden sysctls:
	globForbiddenSysctls := mapset.NewThreadUnsafeSet[string]()
	for _, elem := range settings.ForbiddenSysctls.ToSlice() {
		if before, ok := strings.CutSuffix(elem, "*"); ok {
			globForbiddenSysctls.Add(before)
		}
	}

	data.ForEach(func(_, value gjson.Result) bool {
		sysctl := gjson.Get(value.String(), "name").String()

		if settings.ForbiddenSysctls.Contains(sysctl) {
			err = fmt.Errorf("sysctl %s is on the forbidden list", sysctl)
			return false // stop iterating
		}

		// if sysctl matches a pattern, it is forbidden:
		for _, elem := range globForbiddenSysctls.ToSlice() {
			if strings.HasPrefix(sysctl, elem) {
				if !settings.AllowedUnsafeSysctls.Contains(sysctl) {
					// sysctl is not whitelisted
					err = fmt.Errorf("sysctl %s is on the forbidden list", sysctl)
					return false // stop iterating
				}
			}
		}

		// if sysctl is not on the safe list nor an exception, it is forbidden:
		if !knownSafeSysctls.Contains(sysctl) &&
			!settings.AllowedUnsafeSysctls.Contains(sysctl) {
			err = fmt.Errorf("sysctl %s is not on safe list, nor is in the allowedUnsafeSysctls list",
				sysctl)
			return false // stop iterating
		}
		return true // continue iterating
	})

	if err != nil {
		logger.DebugWithFields("rejecting pod object", func(e onelog.Entry) {
			name := gjson.GetBytes(payload, "request.object.metadata.name").String()
			namespace := gjson.GetBytes(payload, "request.object.metadata.namespace").String()
			e.String("name", name)
			e.String("namespace", namespace)
		})

		return kubewarden.RejectRequest(
			kubewarden.Message(err.Error()),
			kubewarden.NoCode)
	}

	return kubewarden.AcceptRequest()
}
