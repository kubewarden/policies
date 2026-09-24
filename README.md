[![Artifact Hub](https://img.shields.io/endpoint?url=https://artifacthub.io/badge/repository/kubewarden)](https://artifacthub.io/packages/search?kind=13&verified_publisher=true&official=true&cncf=true&sort=relevance&page=1)

# Kubewarden Policies

Welcome to the central repository for [Kubewarden](https://www.kubewarden.io/)
Policies. This repository contains a collection of curated, ready-to-use
policies designed to secure and govern your Kubernetes clusters using
WebAssembly (Wasm). This repository acts as a marketplace/monorepo for policies
that address common security and compliance needs.

# Maintenance and distribution

All policies contained within this monorepo are officially maintained by the
Kubewarden team. We ensure that these policies are kept up-to-date with the
latest Kubernetes API changes and security best practices.

You can browse, search, and view detailed documentation for all these policies
on Artifact Hub. This is the easiest way to discover policy capabilities,
configuration parameters, and version history.

View the official collection here: 👉 Artifact Hub: [Kubewarden
Policies](https://artifacthub.io/packages/search?kind=13&verified_publisher=true&official=true&cncf=true&sort=relevance&page=1)

# Released Policies and Staging Policies

The `policies/` directory holds the released policies. The CI builds, tests and
releases them.

The `staging/` directory holds Rego policies that are not ready for release.
The CI ignores this directory. A policy moves to `policies/` when it is ready.
The [CONTRIBUTING.md](./CONTRIBUTING.md) guide describes that move.

# How to Use a Policy

If you want to try the policies from the source code, you can follow these
step:

## Building a Policy

Navigate to the specific policy directory you wish to build and run the make
command:

```console
cd policies/<policy-name>
make
```

This will compile the source code into a `policy.wasm` file, located in the
policy directory

## Local Testing with kwctl

You can test a policy against a local Kubernetes resource (in JSON or YAML
format) without a cluster using `kwctl`:

```console
make annotated-policy.wasm
kwctl run --request-path request.json --settings-json '{}' annotated-policy.wasm
```

All policies under this repository have tests. Therefore, instead of calling
directly `kwctl` commands, users can test policies changes with the Makefile
targets available:

```console
make test e2e-tests
```

For more information about how to develop and change policies, refer to the
[CONTRIBUTING.md](./CONTRIBUTING.md) guide
