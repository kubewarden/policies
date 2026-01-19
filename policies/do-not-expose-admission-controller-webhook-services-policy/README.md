[![Stable](https://img.shields.io/badge/status-stable-brightgreen?style=for-the-badge)](https://github.com/kubewarden/community/blob/main/REPOSITORIES.md#stable)

This policy identifies Kubernetes Services that are:

- Exposed externally via Ingress resources, NodePort services, or LoadBalancer services.
- Used internally by [Dynamic Admission Controllers](https://kubernetes.io/docs/reference/access-authn-authz/extensible-admission-controllers/) as webhook endpoints.

Exposing webhook endpoints externally increases the attack surface,
as highlighted by [CVE-2025-1974](https://kubernetes.io/blog/2025/03/24/ingress-nginx-cve-2025-1974/). This policy helps secure your cluster by detecting such
misconfiguration.

### How It Works

1. The policy scans all services referenced by `ValidatingWebhookConfiguration`
   and `MutatingWebhookConfiguration`.
2. It queries the Kubernetes API to identify services exposed externally via
   `Ingress` resources, or via NodePort or LoadBalancer services.
3. Any misconfigured `(Validating|Mutating)WebhookConfiguration` is identified.

## Settings

This policy has no configurable settings.

## Access to Kubernetes resources

The policy requires access to the Kubernetes API to query Ingress resources.
This makes it a "[context-aware policy](https://docs.kubewarden.io/reference/spec/context-aware-policies)".

### Deployment

Deploy the policy as a `ClusterAdmissionPolicy` with the `contextAwareResources` field properly set. Use the following command to scaffold the policy:

```console
kwctl scaffold manifest --type ClusterAdmissionPolicy --allow-context-aware <policy name>
```
