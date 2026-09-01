package main

import (
	"encoding/json"
	"errors"
	"fmt"
	"net/http"
	"strings"

	onelog "github.com/francoispqt/onelog"
	corev1 "github.com/kubewarden/k8s-objects/api/core/v1"
	kubewarden "github.com/kubewarden/policy-sdk-go"
	kubewarden_protocol "github.com/kubewarden/policy-sdk-go/protocol"
)

func validate(payload []byte) ([]byte, error) {
	// Create a ValidationRequest instance from the incoming payload
	validationRequest := kubewarden_protocol.ValidationRequest{}
	err := json.Unmarshal(payload, &validationRequest)
	if err != nil {
		return kubewarden.RejectRequest(
			kubewarden.Message(err.Error()),
			kubewarden.Code(http.StatusBadRequest))
	}

	settings := Settings{}
	if err = json.Unmarshal(validationRequest.Settings, &settings); err != nil {
		return kubewarden.RejectRequest(
			kubewarden.Message(err.Error()),
			kubewarden.Code(http.StatusBadRequest))
	}

	if len(settings.AllowedHostPaths) == 0 {
		// empty settings, accepting
		return kubewarden.AcceptRequest()
	}

	podSpec, err := kubewarden.ExtractPodSpecFromObject(validationRequest)
	if err != nil {
		return kubewarden.RejectRequest(
			kubewarden.Message(err.Error()),
			kubewarden.Code(http.StatusBadRequest))
	}

	logger.DebugWithFields("validating pod object", func(e onelog.Entry) {
		e.String("name", validationRequest.Request.Name)
		e.String("namespace", validationRequest.Request.Namespace)
	})

	volumes := make([]*corev1.Volume, 0)
	for _, volume := range podSpec.Volumes {
		if volume.HostPath != nil {
			volumes = append(volumes, volume)
		}
	}

	volumeMounts := make([]*corev1.VolumeMount, 0)
	volumeMounts = append(volumeMounts, getVolumeMounts(podSpec.InitContainers)...)
	volumeMounts = append(volumeMounts, getVolumeMounts(podSpec.Containers)...)

	if err = checkVolumeHostPaths(volumes, volumeMounts, settings); err != nil {
		logger.DebugWithFields("rejecting pod object", func(e onelog.Entry) {
			e.String("name", validationRequest.Request.Name)
			e.String("namespace", validationRequest.Request.Namespace)
		})
		return kubewarden.RejectRequest(
			kubewarden.Message(err.Error()),
			kubewarden.NoCode)
	}

	return kubewarden.AcceptRequest()
}

// checkVolumeHostPaths validates that every hostPath volume, mounted by one
// of the given volumeMounts, matches one of the settings.AllowedHostPaths
// entries. It returns an aggregated error describing every violation found.
func checkVolumeHostPaths(volumes []*corev1.Volume, volumeMounts []*corev1.VolumeMount, settings Settings) error {
	var err error

	for _, volume := range volumes {
		for _, mount := range volumeMounts {
			if *volume.Name != *mount.Name {
				// volume and mount don't match, skip
				continue
			}
			err = errors.Join(err, checkMountAgainstAllowedHostPaths(volume, mount, settings))
		}
	}

	return err
}

// checkMountAgainstAllowedHostPaths validates a single volume/mount pair
// against the settings.AllowedHostPaths entries.
func checkMountAgainstAllowedHostPaths(volume *corev1.Volume, mount *corev1.VolumeMount, settings Settings) error {
	match := false
	var errsMount error // all errors of current mount
	// readOnly attribute of most specific AllowedHostPath takes precendence:
	previousAllowedHostPath := ""
	for _, allowedHostPath := range settings.AllowedHostPaths {
		if !hasPathPrefix(*volume.HostPath.Path, allowedHostPath.PathPrefix) {
			continue
		}
		if !hasPathPrefix(allowedHostPath.PathPrefix, previousAllowedHostPath) {
			continue
		}
		// allowedHostPath is more specific (and has precendence over
		// past allowedHostPath), or the same path
		match = true
		validationError := validatePath(
			*volume.HostPath.Path,
			*mount.Name,
			mount.ReadOnly,
			allowedHostPath,
		)
		// build all errors for this mount:
		if validationError == nil {
			// drop errors in errsMount, we found a more
			// specific path that validates the current
			// mount
			errsMount = nil
		} else {
			// we found even more errors for this specific mount, append
			errsMount = errors.Join(errsMount, validationError)
		}
		previousAllowedHostPath = allowedHostPath.PathPrefix
	}

	if !match {
		// path didn't match against any PathPrefix in settings
		errsMount = errors.Join(
			errsMount,
			fmt.Errorf("hostPath '%s' mounted as '%s' is not in the AllowedHostPaths list",
				*volume.HostPath.Path, *mount.Name),
		)
	}

	return errsMount
}

// validatePath validates the path prefix and its readOnly state against the
// passed hostPath, and returns a matching error if failed.
func validatePath(path, mountName string, readOnly bool, hostPath HostPath) error {
	if hasPathPrefix(path, hostPath.PathPrefix) {
		if readOnly != hostPath.ReadOnly {
			return fmt.Errorf("hostPath '%s' mounted as '%s' should be readOnly '%t'",
				path, mountName, hostPath.ReadOnly)
		}
	}
	return nil
}

func hasPathPrefix(path string, prefix string) bool {
	// allow "/foo", "/foo/", "/foo/bar", etc
	// disallow "/fool", "/etc/foo", etc
	// "/foo/../" is never valid.
	// Hence, ensure paths terminate in `/`:
	pathTerminated := path
	if !strings.HasSuffix(pathTerminated, "/") {
		pathTerminated += "/"
	}
	prefixTerminated := prefix
	if !strings.HasSuffix(prefixTerminated, "/") {
		prefixTerminated += "/"
	}
	return strings.HasPrefix(pathTerminated, prefixTerminated)
}

func getVolumeMounts(containers []*corev1.Container) []*corev1.VolumeMount {
	volumeMounts := make([]*corev1.VolumeMount, 0)
	for _, container := range containers {
		volumeMounts = append(volumeMounts, container.VolumeMounts...)
	}
	return volumeMounts
}
