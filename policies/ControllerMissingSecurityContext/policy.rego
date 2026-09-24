package policy

import future.keywords.in

default exclude_namespaces := ["kube-system"]
default exclude_label_key := ""
default exclude_label_value := ""

exclude_namespaces := input.parameters.exclude_namespaces
exclude_label_key := input.parameters.exclude_label_key
exclude_label_value := input.parameters.exclude_label_value

violation[result] {
	isExcludedNamespace == false
	not exclude_label_value == controller_input.metadata.labels[exclude_label_key]
	not has_security_context(controller_spec)	# Pod securityContext missing or empty
	some i
	containers := controller_spec.containers[i]
	not has_security_context(containers)	# Container securityContext missing or empty
	result = {
		"issue_detected": true,
		"msg": sprintf("Container missing spec.template.spec.containers[%v].securityContext while the Pod spec.template.spec.securityContext is missing or empty.", [i]),
		"violating_key": "spec.template.spec.containers[%v]",
	}
}

# The API server defaults an absent securityContext to an empty object. An
# empty object carries no security setting, so it counts as missing.
has_security_context(spec) {
	count(spec.securityContext) > 0
}

controller_input = input.review.object

controller_spec = controller_input.spec.template.spec {
	contains(controller_input.kind, {"StatefulSet", "DaemonSet", "Deployment", "Job", "ReplicaSet"})
} else = controller_input.spec {
	controller_input.kind == "Pod"
} else = controller_input.spec.jobTemplate.spec.template.spec {
	controller_input.kind == "CronJob"
}

contains(kind, kinds) {
	kinds[_] = kind
}

isExcludedNamespace = true {
	controller_input.metadata.namespace
	controller_input.metadata.namespace in exclude_namespaces
} else = false
