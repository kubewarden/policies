# Contributing to Kubewarden Policies

Thank you for your interest in contributing! This document outlines the
technical workflow for developing, testing, and releasing policies within this
repository.

Kubewarden is language-agnostic. This repository contains policies written in
Rust, Go and Rego.

# Directory Structure

This repository is a monorepo. While each policy is functionally independent,
they share common tooling and dependencies.

```text
.
├── policies/ # Policy source code
│ ├── Cargo.toml # Rust Workspace configuration
│ ├── Cargo.lock # Shared dependency lock file for Rust
│ ├── Makefile.p-rego # Shared build rules of the Rego policies
│ ├── <policy-name>/ # Specific policy directory
│ │ ├── src/ # Source code
│ │ ├── test_data/ # Files for testing
│ │ ├── Makefile # Standardized build commands
│ │ ├── metadata.yml# Artifact Hub metadata
│ │ ├── <any other policy file>
│ ├── <policy-name>/ # Specific policy directory
│ │ ├── <any other policy file>
├── staging/ # Rego policies that are not ready for release
│ ├── <policy-name>/ # Specific policy directory
```

# Rego Policies

Rego policies have Rego unit tests files in two layouts, the `Makefile.p-rego`
supports both:
- in `policy_test.rego` next to `policy.rego`
- one or more files under `tests/`

The `e2e-tests` target runs `bats e2e.bats`. Every Rego policy has that file.

## End-to-End Tests of a Rego Policy

`hack/generate-rego-e2e.py` writes the end-to-end test of a Rego policy from
its unit tests. Each `test_*` rule holds the settings and the reviewed object,
so the script can build the fixtures:

```console
hack/generate-rego-e2e.py policies/<policy-name>
hack/generate-rego-e2e.py --all
```

It writes `test_data/<case>.json`, `test_data/<case>-settings.json` and
`e2e.bats`, with one accept case and one reject case. It builds the policy and
runs `kwctl` once per case, to make sure that the request gives the same
verdict as the unit test and to get the message that the test asserts.

Notes:

- The script needs `opa`, `kwctl`, `make` and `bats`.
- A generated `e2e.bats` carries a marker line. The script does not overwrite a
  file without that marker, so a hand-written test is safe. Use `--force` to
  overwrite one.
- To cover a verdict that the policy does not test yet, add the missing unit
  test and run the script again. Do not write the fixture by hand.
- When you reword the message of a policy, run the script again for that
  policy. The message is part of the assertion.

# The staging Directory

The `staging/` directory holds Rego policies that are not ready for release.
The CI ignores this directory. It calculates the policy matrix from `policies/`
only, so a change under `staging/` starts no build, no test and no release.

To promote a policy, move its directory to `policies/`, add the `Makefile`
symbolic link to `../Makefile.p-rego`, and add a
`.github/release-drafter-<policy-name>.yml` file with:

```console
go run hack/release-drafter-config-generator.go \
    --policy-name <policy-name> \
    --output .github/release-drafter-<policy-name>.yml
```

# Rust Workspace

To optimize build times and ensure consistency, all Rust policies are members
of a single Rust Workspace. The `policies/Cargo.toml` defines the workspace
members. Common dependencies are shared across policies to reduce maintenance
overhead. When adding a new Rust policy, ensure it is added to the members list
in the root Cargo.toml.

# How to Build Policies

We use `make` to provide a consistent interface across different programming
languages.

Navigate to the policy directory:

```console
cd policies/policy-name
```

Build the Wasm binary:

```console
    make policy.wasm
```

# How to Test Policies

All policies in this repository should provide unit test and integration tests.
To run all the tests run the following command:

```console
make test e2e-tests
```

## Language-Scoped Makefile Targets

The root `Makefile` provides targets that operate across all policies
regardless of language (`test`, `lint`, `e2e-tests`). For faster iteration,
language-scoped variants are also available.

| Target           | Scope                                                                             |
| ---------------- | --------------------------------------------------------------------------------- |
| `test-rust`      | Rust policies (detected by `Cargo.toml`) + shared crates under `policies/crates/` |
| `test-go`        | Go policies (detected by `go.mod`)                                                |
| `test-rego`      | Rego policies (detected by a `.rego` file)                                       |
| `lint-rust`      | Rust policies + shared crates under `policies/crates/`                            |
| `lint-go`        | Go policies                                                                       |
| `lint-rego`      | Rego policies                                                                     |
| `e2e-tests-rust` | Rust policies                                                                     |
| `e2e-tests-go`   | Go policies                                                                       |
| `e2e-tests-rego` | Rego policies                                                                     |

The language detection is file-based: a policy directory is considered Rust if
it contains a `Cargo.toml`, Go if it contains a `go.mod`, and Rego if it
contains a `.rego` file. These sets are mutually exclusive. The shared crates
under `policies/crates/` are all Rust and are included in the `*-rust` targets
for `test` and `lint` (consistent with the full-repo targets), but not for
`e2e-tests` since crates have no end-to-end tests.

# How to Release a Policy

Every release starts from a git tag with the pattern `<policy-name>/v<version>`.
A tag comes from one of two places:

- The merge of a release PR. The CI opens this PR.
- A maintainer, by hand. This is the path for the first release of a policy.

## Automated releases

The workflow `.github/workflows/trigger-policy-release.yaml` runs on a
schedule, and on demand. For each policy, it reads the version from
`metadata.yml` and checks for the tag `<policy-name>/v<version>`:

- If the tag exists, release-drafter computes the next version from the PRs
  merged since that tag. Then updatecli opens a PR that bumps the version in
  all required files. The PR has the `TRIGGER-RELEASE` label.
- If the tag does not exist, the workflow skips the policy and prints a
  notice. The policy was never released from this repository. See
  [First release of a policy](#first-release-of-a-policy).

When the release PR is merged, `.github/workflows/release-tag.yaml` creates
the tag and starts `.github/workflows/release.yml`. That workflow builds,
signs and publishes the policy.

To run the workflow for one policy, with a version of your choice:

```console
gh workflow run trigger-policy-release.yaml \
    -f "policy-working-dir=allowed-proc-mount-types-psp-policy" \
    -f "policy-version=1.0.6" \
    -R kubewarden/policies
```

> [!IMPORTANT]
> The `policy-working-dir` must be the name of the directory under the
> `policies` directory

The `policy-version` must be greater than the version in `metadata.yml`. If you
omit `policy-version`, release-drafter computes the next version.

> [!NOTE]
> The `trigger-policy-release.yaml` CI can also be trigged in the Github UI.

## First release of a policy

A new policy, or a policy imported from another repository, has a version in
`metadata.yml` with no tag. The automation cannot compute a next version
without a previous tag, so a maintainer makes the first release by hand:

1. Make sure that `main` has the version that you want to release in
   `metadata.yml`, and in `Cargo.toml` for a Rust policy.
2. Create the tag on `main` and push it:

   ```console
   git tag -a <policy-name>/v<version> -m "Release version <policy-name>/v<version>"
   git push origin <policy-name>/v<version>
   ```

3. The push starts `release.yml`. The run fails if the version in the tag
   does not match the version in `metadata.yml`.

From the next version on, the automation opens release PRs for the policy.

> [!WARNING]
> Do not push tags from another repository when you import a policy. Each
> tag push starts a release, and the tag commits do not exist in this
> repository. Make the first release by hand, with a version greater than
> the last version released elsewhere.

## Release flow

```mermaid
flowchart TD

A[trigger-policy-release.yaml runs] --> B{Tag for current version exists}

B -->|Yes| C[Open PR that bumps the version]
B -->|No| D[Skip policy, print notice]
C --> E[PR merged]
E --> F[release-tag.yaml creates the tag]
F --> G[release.yml builds and publishes]
H[Maintainer pushes a tag by hand] --> G
```

## Publish the 'latest' tag of one policy

You can also run `release.yml` manually, against a branch. That run builds one
policy and publishes it with the `:latest` tag:

```console
gh workflow run release.yml \
    -f "policy-working-dir=allowed-proc-mount-types-psp-policy" \
    -R kubewarden/policies
```

The version comes from the `io.kubewarden.policy.version` annotation of
`metadata.yml`. A run against a branch only pushes the `:latest` OCI tag. It
creates no git tag and no GitHub release. It updates neither ArtifactHub nor
the policy catalog.

> [!NOTE]
> A run of `release.yml` against a tag is a normal release. The policy name
> and the version come from the tag name, and the run ignores the
> `policy-working-dir` input.
>
> `release-tag.yaml` starts this kind of run. GitHub fires no workflow for a
> tag pushed with `GITHUB_TOKEN`, so `release-tag.yaml` must start the run
> itself, with `gh workflow run release.yml --ref <tag>`.

# Tag Pattern

The CI creates tags using the following logic based on the subdirectory under
the `policies` directory modified:

```
<policy-subdirectory-name>/v<semantic-version>`
```

Example: If you update the `pod-privileged-policy` policy to version `0.1.5`,
the CI will generate the tag: `pod-privileged-policy/v0.1.5`

# OCI Namespaces

Each policy declares its OCI URL in the `io.kubewarden.policy.ociUrl` annotation
of `metadata.yml`. All policies of this repository use the `policies` namespace:

```
ghcr.io/kubewarden/policies/<policy-name>
```

The CI reads only the last segment of the annotation, `<policy-name>`.
The CI then builds the OCI URL from a base and the `POLICIES_OCI_BASE`
repository variable:

```bash
<POLICIES_OCI_BASE>/<policy-name>
```

When `POLICIES_OCI_BASE` is empty, the base is
`ghcr.io/<repository-owner>/policies`.

Therefore:

- The registry and the namespace of the annotation have no effect. Keep them at
  `ghcr.io/kubewarden/policies` so that the annotation shows the true location
  of the policy of the upstream repository.
- Upstream repository publishes to `ghcr.io/kubewarden/policies`
- A fork publishes to its own registry. It never publishes to `kubewarden`.

To keep the annotation and the push target in agreement, the CI stops with an
error when:

- the annotation is absent, or
- the namespace of the annotation is not `policies`.

The `set-policy-oci-url` action then writes the calculated URL into the
annotation of the checked out `metadata.yml`. The action does not commit this
change. It runs in the `release` job before `kwctl annotate`, and in the
`push-artifacthub` job before `kwctl scaffold artifacthub`.

> [!IMPORTANT]
> `kwctl annotate` writes the annotation into the Wasm module, `kwctl push`
> uses it as the push target, and `kwctl scaffold artifacthub` writes it into
> `artifacthub-pkg.yml`. All three read the same value, so the URL that
> ArtifactHub shows is the URL from which users can pull the policy.

## The OCI tests/ namespace

The `ghcr.io/kubewarden/tests/<policy-name>` namespace is reserved for manual
pushes. The CI never writes to it. The policies in this namespace are used by
integration tests and the like.

# Forks

A fork releases the policies to its own registry. It does not need a change of
the files that this repository tracks. Thus a fork stays easy to synchronize
with the upstream repository.

## Forks on GitHub

A fork on GitHub needs no configuration. The CI publishes to
`ghcr.io/<repository-owner>/policies/<policy-name>` with the token of the
workflow.

## Forks that use another registry

Give the fork one repository variable and two repository secrets:

| Name                    | Type     | Example                                |
| ----------------------- | -------- | -------------------------------------- |
| `POLICIES_OCI_BASE`     | variable | `registry.example.com/team/policies`   |
| `POLICIES_OCI_USERNAME` | secret   | `robot$policies`                       |
| `POLICIES_OCI_PASSWORD` | secret   | the password or the token of that user |

The value of `POLICIES_OCI_BASE` is the registry, the organization and the
namespace, without the name of the policy. The CI adds the name of the policy
and the tag.

When `POLICIES_OCI_BASE` is empty, the CI ignores the two secrets and it logs
in to GHCR with the token of the workflow.

## What a fork must know

- The `io.kubewarden.policy.url` and `io.kubewarden.policy.source` annotations
  still point to the upstream repository. Therefore the link to the GitHub
  release in `artifacthub-pkg.yml` also points to the upstream repository.
- The `push-artifacthub` job runs on a fork. It writes to the `artifacthub`
  branch of the fork only.
- The `release-catalog` job runs on the upstream repository only.

## Signatures on a fork

`cosign` signs the policies of a fork with the GitHub Actions identity of that
fork. Three results come from this:

- The signature goes to the registry of the fork, next to the policy.
- The certificate comes from the public Fulcio, and the entry goes to the
  public Rekor. The digest of the policy and the name of the fork become
  public, also when the registry is private.
- Users must verify with the identity of the fork:

```console
cosign verify \
  --certificate-identity-regexp 'https://github.com/<owner>/<repo>/.github/workflows/release.yml@.*' \
  --certificate-oidc-issuer https://token.actions.githubusercontent.com \
  <base>/<policy-name>:<tag>
```
