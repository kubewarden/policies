#!/bin/bash
# Generates the Updatecli values file that drives the weekly Hauler manifest
# update (updatecli/update-hauler-manifest.yaml). The published-policy list,
# the OCI registry path, and the release-workflow identity URL are all
# derived at generation time, so the same script produces correct values
# whether run against the canonical repository or a fork.
#
# The manifest lives on its own branch, with no other file. The automation
# commits to that branch directly and opens no pull request. See
# "Hauler Manifest" in CONTRIBUTING.md.
#
# Usage: generate-hauler-values.sh [--owner OWNER] [--repo REPO]
#          [--target-owner OWNER] [--target-repo REPO]
#          [--target-branch BRANCH] [--output PATH]
#   --owner          GitHub/GHCR owner of the policy images. Default: kubewarden
#   --repo           GitHub repository name of the policy images. Default: policies
#   --target-owner   Owner of the repository that holds the manifest branch.
#                     Default: empty (the policy reads GITHUB_REPOSITORY at run time)
#   --target-repo    Name of the repository that holds the manifest branch.
#                     Default: empty (the policy reads GITHUB_REPOSITORY at run time)
#   --target-branch  Branch that holds only the Hauler manifest. Default: hauler-manifest
#   --output         Where to write the values file. Default: - (stdout)
#
# `--owner`/`--repo` only affect the image path and the
# `certificate-identity-regexp`; they point at the registry the policy images
# are published to. `--target-owner`/`--target-repo` point at the repository
# the automation commits to, and are independent of `--owner`/`--repo`: a fork
# can read policy versions from the upstream `kubewarden` registry while
# committing to its own repository. Leave `--target-owner`/`--target-repo`
# unset to let the policy derive them from `GITHUB_REPOSITORY`, which GitHub
# Actions sets to the repository running the workflow.

set -euo pipefail

usage() {
  echo "Usage: $0 [--owner OWNER] [--repo REPO] [--target-owner OWNER]" >&2
  echo "          [--target-repo REPO] [--target-branch BRANCH] [--output PATH]" >&2
  exit 1
}

OWNER="kubewarden"
REPO="policies"
TARGET_OWNER=""
TARGET_REPO=""
TARGET_BRANCH="hauler-manifest"
OUTPUT="-"

while [ "$#" -gt 0 ]; do
  case "$1" in
  --owner)
    [ "$#" -ge 2 ] || usage
    OWNER="$2"
    shift 2
    ;;
  --repo)
    [ "$#" -ge 2 ] || usage
    REPO="$2"
    shift 2
    ;;
  --target-owner)
    [ "$#" -ge 2 ] || usage
    TARGET_OWNER="$2"
    shift 2
    ;;
  --target-repo)
    [ "$#" -ge 2 ] || usage
    TARGET_REPO="$2"
    shift 2
    ;;
  --target-branch)
    [ "$#" -ge 2 ] || usage
    TARGET_BRANCH="$2"
    shift 2
    ;;
  --output)
    [ "$#" -ge 2 ] || usage
    OUTPUT="$2"
    shift 2
    ;;
  *)
    usage
    ;;
  esac
done

# A direct commit with no pull request (see the "scm:" block in render()
# below) must never land on a development branch: nothing reviews it before
# it takes effect.
case "$TARGET_BRANCH" in
main | master)
  echo "ERROR: --target-branch must not be 'main' or 'master'. This automation" >&2
  echo "       commits directly to the target branch, with no pull request." >&2
  exit 1
  ;;
esac

if ! command -v yq >/dev/null 2>&1; then
  echo "ERROR: 'yq' is required" >&2
  exit 1
fi

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
EXCLUSION_LIST="$REPO_ROOT/policies/excluded-from-publishing.txt"
if [ ! -f "$EXCLUSION_LIST" ]; then
  echo "ERROR: expected file '$EXCLUSION_LIST' not found" >&2
  exit 1
fi

is_excluded() {
  local dir="$1"
  local entry
  while IFS= read -r line || [ -n "$line" ]; do
    entry="$(echo "$line" | sed -e 's/#.*//' -e 's/[[:space:]]//g')"
    [ -z "$entry" ] && continue
    [ "$entry" = "$dir" ] && return 0
  done <"$EXCLUSION_LIST"
  return 1
}

# Publishable set = every policy directory (contains a
# metadata.yml/metadata.yaml, maxdepth 2, excluding policies/crates/) minus
# the exclusion list.
mapfile -t POLICY_DIRS < <(find "$REPO_ROOT/policies" -maxdepth 2 \( -name metadata.yml -o -name metadata.yaml \) -print0 |
  while IFS= read -r -d '' metadata; do basename "$(dirname "$metadata")"; done | LC_ALL=C sort -u)

declare -a PAIRS=()
for dir in "${POLICY_DIRS[@]}"; do
  is_excluded "$dir" && continue

  METADATA_FILE="$(find "$REPO_ROOT/policies/$dir" -maxdepth 1 \( -name 'metadata.yml' -o -name 'metadata.yaml' \) -print -quit)"
  if [ -z "$METADATA_FILE" ]; then
    echo "ERROR: no metadata.yml/metadata.yaml found in 'policies/$dir'" >&2
    exit 1
  fi

  policy_id="$(yq -r '.annotations."io.kubewarden.policy.ociUrl" // ""' "$METADATA_FILE")"
  policy_id="${policy_id##*/}"
  if [ -z "$policy_id" ] || [ "$policy_id" = "null" ]; then
    echo "ERROR: 'policies/$dir' has no io.kubewarden.policy.ociUrl annotation but is not" >&2
    echo "       listed in $EXCLUSION_LIST. Add the annotation, or add the policy to that" >&2
    echo "       file if it should not be published." >&2
    exit 1
  fi

  PAIRS+=("$policy_id"$'\t'"$dir")
done

if [ "${#PAIRS[@]}" -eq 0 ]; then
  echo "ERROR: no publishable policies found (everything is either missing a" >&2
  echo "       metadata.yml/metadata.yaml or listed in $EXCLUSION_LIST)" >&2
  exit 1
fi

# Sort by policy id, not directory name, matching the order the hand-written
# values file used to have.
mapfile -t PAIRS < <(printf '%s\n' "${PAIRS[@]}" | LC_ALL=C sort)

declare -a IDS=()
declare -a DIRS=()
for pair in "${PAIRS[@]}"; do
  IDS+=("${pair%%$'\t'*}")
  DIRS+=("${pair#*$'\t'}")
done

# The generated values key `versions:` and `documents[].items[]` by policy
# id. Two directories sharing an id would silently collapse into one entry,
# so fail loudly instead of producing a manifest with a wrong versionFrom.
declare -A SEEN_DIR_FOR_ID=()
for i in "${!IDS[@]}"; do
  id="${IDS[$i]}"
  dir="${DIRS[$i]}"
  if [ -n "${SEEN_DIR_FOR_ID[$id]:-}" ]; then
    echo "ERROR: policy id '$id' is used by both 'policies/${SEEN_DIR_FOR_ID[$id]}'" >&2
    echo "       and 'policies/$dir'. Each io.kubewarden.policy.ociUrl must be unique." >&2
    exit 1
  fi
  SEEN_DIR_FOR_ID[$id]="$dir"
done

render() {
  cat <<EOF
# Generated by hack/generate-hauler-values.sh for owner=$OWNER repo=$REPO,
# target branch=$TARGET_BRANCH.
# DO NOT EDIT BY HAND. Regenerate with: make hauler-values
pipelineid: "update_hauler_manifest"

hauler:
  file: "hauler_manifest.yaml"
  createIfMissing: true

# One dockerimage source per published Kubewarden policy. The version is
# read from the OCI registry (ghcr.io), not from the policy's metadata.yml,
# so the Hauler manifest always reflects what is actually published.
versions:
EOF
  for id in "${IDS[@]}"; do
    cat <<EOF
  $id:
    kind: dockerimage
    image: "ghcr.io/$OWNER/policies/$id"
    versionfilter:
      kind: semver
EOF
  done

  cat <<EOF

documents:
  - name: kubewarden-policies
    kind: Images
    apiVersion: "content.hauler.cattle.io/v1"
    createIfMissing: true
    sort: true
    prune: true
    annotations:
      hauler.dev/certificate-oidc-issuer: "https://token.actions.githubusercontent.com"
    items:
EOF
  for i in "${!IDS[@]}"; do
    id="${IDS[$i]}"
    dir="${DIRS[$i]}"
    cat <<EOF
      - repository: ghcr.io/$OWNER/policies/$id
        versionFrom: $id
        fields:
          certificate-identity-regexp: "https://github.com/$OWNER/$REPO/.github/workflows/release.yml@refs/tags/$dir/{version}"
EOF
  done

  cat <<EOF

# The manifest branch holds only the Hauler manifest, so the automation
# commits to it directly instead of opening a pull request. See "Commit to a
# branch without a pull request" in the hauler/manifest policy README.
scm:
  enabled: true
  kind: "github"
EOF

  if [ -n "$TARGET_OWNER" ] && [ -n "$TARGET_REPO" ]; then
    cat <<EOF
  owner: "$TARGET_OWNER"
  repository: "$TARGET_REPO"
EOF
  else
    cat <<EOF
  # owner/repository are deliberately omitted. The hauler/manifest policy
  # derives them from GITHUB_REPOSITORY at run time, so the automation
  # commits to whichever repository the workflow runs in (a fork when
  # testing, kubewarden/policies upstream). They must not be tied to
  # \$OWNER/\$REPO above, which point at the registry the policy images
  # are published to, not at the repository the manifest branch lives in.
EOF
  fi

  cat <<EOF
  env_token: "UPDATECLI_GITHUB_TOKEN"
  branch: "$TARGET_BRANCH"
  workingbranch: false
  force: false
  user: "Kubewarden bot"
  email: "cncf-kubewarden-maintainers@lists.cncf.io"
  commitusingapi: true
  commitmessage:
    type: "chore"
    scope: "deps"
    title: "Update Hauler manifest with published policy versions"
    footers: "Signed-off-by: Kubewarden bot <cncf-kubewarden-maintainers@lists.cncf.io>"

pr:
  enabled: false

pipeline:
  labels:
    ecosystem: "hauler"
    policy: "manifest"
EOF
}

if [ "$OUTPUT" = "-" ]; then
  render
else
  mkdir -p "$(dirname "$OUTPUT")"
  render >"$OUTPUT"
  echo "Wrote $OUTPUT" >&2
fi
