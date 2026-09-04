package main

import (
	"errors"
	"fmt"
	"net/http"

	onelog "github.com/francoispqt/onelog"
	"github.com/kubewarden/gjson"
	kubewarden "github.com/kubewarden/policy-sdk-go"
)

func validate(payload []byte) ([]byte, error) {
	settings, err := NewSettingsFromValidationReq(payload)
	if err != nil {
		return kubewarden.RejectRequest(
			kubewarden.Message(err.Error()),
			kubewarden.Code(http.StatusBadRequest))
	}

	if settings.AllowedTypes.Cardinality() == 0 {
		// empty AllowedType list, rejecting
		return kubewarden.RejectRequest(
			kubewarden.Message("No volume type is allowed"),
			kubewarden.NoCode)
	}

	if (settings.AllowedTypes.Cardinality() == 1) &&
		settings.AllowedTypes.Contains("*") {
		// all volume types accepted
		return kubewarden.AcceptRequest()
	}

	volumes := gjson.GetBytes(
		payload,
		"request.object.spec.volumes")
	if !volumes.Exists() {
		// pod defines no volumes, accepting
		return kubewarden.AcceptRequest()
	}

	logger.DebugWithFields("validating pod object", func(e onelog.Entry) {
		name := gjson.GetBytes(payload, "request.object.metadata.name").String()
		namespace := gjson.GetBytes(payload,
			"request.object.metadata.namespace").String()
		e.String("name", name)
		e.String("namespace", namespace)
	})

	if err = checkVolumeTypes(payload, volumes, &settings); err != nil {
		logger.DebugWithFields("rejecting pod object", func(e onelog.Entry) {
			name := gjson.GetBytes(payload, "request.object.metadata.name").String()
			namespace := gjson.
				GetBytes(payload, "request.object.metadata.namespace").String()
			e.String("name", name)
			e.String("namespace", namespace)
		})
		return kubewarden.RejectRequest(
			kubewarden.Message(err.Error()),
			kubewarden.NoCode)
	}

	return kubewarden.AcceptRequest()
}

// checkVolumeTypes verifies that every volume defined by the pod uses one of
// the allowed volume types, returning an aggregated error describing every
// violation found.
func checkVolumeTypes(payload []byte, volumes gjson.Result, settings *Settings) error {
	// Collect volume names used by initContainers and containers
	initContainerVolumeNames := map[string]struct{}{}
	containerVolumeNames := map[string]struct{}{}
	if settings.IgnoreInitContainersVolumes {
		initContainers := gjson.GetBytes(payload, "request.object.spec.initContainers")
		initContainerVolumeNames = getVolumeMountNames(initContainers)

		containers := gjson.GetBytes(payload, "request.object.spec.containers")
		containerVolumeNames = getVolumeMountNames(containers)
	}

	var err error

	for _, volume := range volumes.Array() {
		// obtain volumeName, volumeType:
		var volumeName, volumeType string
		volume.ForEach(func(key, value gjson.Result) bool {
			if key.String() == "name" {
				volumeName = value.String()
			} else {
				// must be the type
				volumeType = key.String()
			}
			return true // keep iterating
		})

		if settings.IgnoreInitContainersVolumes {
			_, usedByInitContainer := initContainerVolumeNames[volumeName]
			_, usedByContainer := containerVolumeNames[volumeName]

			if usedByInitContainer && !usedByContainer {
				// Skip volumes that are only used by initContainers and not by containers
				continue
			}
		}

		if !settings.AllowedTypes.Contains(volumeType) {
			errMsg := fmt.Sprintf("volume '%s' of type '%s' is not in the AllowedTypes list",
				volumeName, volumeType)
			if err == nil {
				err = errors.New(errMsg)
			} else {
				err = fmt.Errorf("%w; %s", err, errMsg)
			}
		}
	}

	return err
}

// getVolumeMountNames extracts volume mount names from a list of containers.
func getVolumeMountNames(containers gjson.Result) map[string]struct{} {
	volumeNames := map[string]struct{}{}
	for _, container := range containers.Array() {
		mounts := container.Get("volumeMounts")
		for _, mount := range mounts.Array() {
			name := mount.Get("name").String()
			if name != "" {
				volumeNames[name] = struct{}{}
			}
		}
	}
	return volumeNames
}
