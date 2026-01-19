[![Kubewarden Policy Repository](https://github.com/kubewarden/community/blob/main/badges/kubewarden-policies.svg)](https://github.com/kubewarden/community/blob/main/REPOSITORIES.md#policy-scope)
[![Stable](https://img.shields.io/badge/status-stable-brightgreen?style=for-the-badge)](https://github.com/kubewarden/community/blob/main/REPOSITORIES.md#stable)

# Namespace Label Propagator

This policy ensures that all workloads deployed in the cluster contain specific
labels defined in the namespace where the workload will be executed.

When a resource is created or updated, this policy copies a list of user-specified
labels from the namespace to the object. Labels defined on the namespace take
precedence over labels already defined inside the resource.

This policy is able to set the labels for the following resource kinds: `Pod`,
`ReplicationController`, `Deployment`, `ReplicaSet`, `StatefulSet`, `DaemonSet`,
`Job` and `CronJob`.

When dealing with Kubernetes resources that generate pods, the policy ensures the
special labels are propagated also to them.

## Settings

This policy has a single setting called `propagatedLabels`, which is a list of
strings representing the labels from the namespace definition that should be
propagated to the workloads deployed in the namespace.

For example, the policy configuration would look like this:

```yaml
propagatedLabels:
- cost-center
- field.cattle.io/projectId
```

In this scenario, when a resource is created, the policy ensures that the
`cost-center` and `field.cattle.io/projectId` labels are copied from the
namespace object to the resource itself.

Label propagation only occurs if the desired labels are already set on the namespace.
If a label is not defined in the namespace, it will not be propagated to the workloads

## Limitations

The policy propagates the labels only when a object is created or updated.

### Changes to the Namespace

Adding a new relevant label to a Namespace is not going to cause all the resources
inside of it to be updated. The individual resources will be updated when a UPDATE
action affects them.

Changing the value of a relevant label inside of the Namespace is not going to update
all the resources defined inside of it. The individual resources will be updated when
a UPDATE action affects them.

Removing a relevant label from the Namespace is not going to cause its removal from
all the resources that are already defined inside of it. The resources will retain
this label forever, even if they are processed again by the policy because of a
UPDATE action.

### Changes to the policy settings

Adding a new label to the list of `propagatedLabels` is not going to udpdate all
the resources already defined inside of the Namespace. The individual resources
will be updated when a UPDATE action affects them.

Removing a label from the list of `propagatedLabels` is not going to remove it
from the resources that already exist inside of the Namespace. The resources will
retain this label forever, even if they are processed again by the policy because of a
UPDATE action.
