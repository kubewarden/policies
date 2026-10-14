#!/bin/bash
set -euo pipefail

# Default authors considered "safe" to auto-merge (bots opening automated PRs)
DEFAULT_AUTHORS=("app/kubewarden-auth-token-generator" "app/renovate")

# Function to display usage
usage() {
    echo "Usage: $0 [--repo <owner/repo>] [--author <login>]... [--all] [--dry-run]"
    echo "  --repo <owner/repo>  (Optional) Target repository (e.g., kubewarden/policies)."
    echo "                       Defaults to the repo of the current directory."
    echo "  --author <login>     (Optional, repeatable) Only consider PRs opened by this author."
    echo "                       Defaults to: ${DEFAULT_AUTHORS[*]}"
    echo "  --all                Consider every open, non-draft PR, regardless of author."
    echo "  --dry-run            Print what would be merged/deleted without making changes."
    exit 1
}

# Check for required tools
if ! command -v gh &> /dev/null; then
    echo "Error: 'gh' (GitHub CLI) is not installed."
    exit 1
fi

if ! command -v jq &> /dev/null; then
    echo "Error: 'jq' is not installed."
    exit 1
fi

TARGET_REPO=""
DRY_RUN=false
ALL_AUTHORS=false
AUTHORS=()

while [[ "$#" -gt 0 ]]; do
    case $1 in
        --repo)
            if [[ -n "${2:-}" && ! "$2" =~ ^-- ]]; then
                TARGET_REPO="$2"
                shift
            else
                echo "Error: --repo requires a repository value (e.g., owner/repo)."
                usage
            fi
            ;;
        --author)
            if [[ -n "${2:-}" && ! "$2" =~ ^-- ]]; then
                AUTHORS+=("$2")
                shift
            else
                echo "Error: --author requires a login value."
                usage
            fi
            ;;
        --all)
            ALL_AUTHORS=true
            ;;
        --dry-run)
            DRY_RUN=true
            ;;
        -h|--help)
            usage
            ;;
        *)
            echo "Unknown parameter passed: $1"
            usage
            ;;
    esac
    shift
done

if [[ "$ALL_AUTHORS" == "false" && ${#AUTHORS[@]} -eq 0 ]]; then
    AUTHORS=("${DEFAULT_AUTHORS[@]}")
fi

# Build the optional -R flag array for gh invocations
REPO_FLAG=()
if [[ -n "$TARGET_REPO" ]]; then
    REPO_FLAG=("-R" "$TARGET_REPO")
fi

echo "Fetching open PRs..."
if [[ "$ALL_AUTHORS" == "true" ]]; then
    echo "  Author filter: (none, considering all open non-draft PRs)"
else
    echo "  Author filter: ${AUTHORS[*]}"
fi
echo "  Dry run:       $DRY_RUN"
echo

# Build a jq author filter, e.g.: (.author.login == "a" or .author.login == "b")
build_author_jq() {
    local expr=""
    local a
    for a in "${AUTHORS[@]}"; do
        if [[ -n "$expr" ]]; then
            expr+=" or "
        fi
        expr+=".author.login == \"$a\""
    done
    echo "$expr"
}

JQ_FILTER='.[] | select(.isDraft == false)'
if [[ "$ALL_AUTHORS" == "false" ]]; then
    author_expr=$(build_author_jq)
    JQ_FILTER+=" | select($author_expr)"
fi
JQ_FILTER+=' | [(.number|tostring), .author.login, .headRefName, .mergeable, .title] | join("\t")'

PR_LIST=$(gh pr list "${REPO_FLAG[@]}" --state open --limit 200 \
    --json number,title,author,headRefName,isDraft,mergeable \
    --jq "$JQ_FILTER")

if [[ -z "$PR_LIST" ]]; then
    echo "No matching open PRs found."
    exit 0
fi

# Buckets, tracked as newline-separated "#N title" strings
MERGED_LIST=""
FAILED_CHECKS_LIST=""
PENDING_CHECKS_LIST=""
CONFLICT_LIST=""
UNKNOWN_MERGEABLE_LIST=""
MERGE_ERROR_LIST=""

MERGE_ERROR_OCCURRED=false

while IFS=$'\t' read -r number author head mergeable title; do
    [[ -z "$number" ]] && continue

    echo "--------------------------------------------------"
    echo "PR #$number [$author]: $title"
    echo "  branch:    $head"
    echo "  mergeable: $mergeable"

    if [[ "$mergeable" == "CONFLICTING" ]]; then
        echo "  -> Skipping: PR has merge conflicts."
        CONFLICT_LIST+="#$number $title"$'\n'
        continue
    fi

    if [[ "$mergeable" == "UNKNOWN" ]]; then
        echo "  -> Skipping: mergeability not yet computed by GitHub, retry later."
        UNKNOWN_MERGEABLE_LIST+="#$number $title"$'\n'
        continue
    fi

    # Determine check status. gh pr checks exit codes:
    #   0 = all checks passed
    #   1 = at least one check failed
    #   8 = some checks still pending
    set +e
    gh pr checks "$number" "${REPO_FLAG[@]}" >/dev/null 2>&1
    checks_status=$?
    set -e

    case "$checks_status" in
        0)
            echo "  checks:    green"
            ;;
        1)
            echo "  -> Skipping: one or more checks failed."
            FAILED_CHECKS_LIST+="#$number $title"$'\n'
            continue
            ;;
        8)
            echo "  -> Skipping: checks still pending."
            PENDING_CHECKS_LIST+="#$number $title"$'\n'
            continue
            ;;
        *)
            echo "  -> Skipping: unable to determine check status (gh exit code $checks_status)."
            PENDING_CHECKS_LIST+="#$number $title"$'\n'
            continue
            ;;
    esac

    if [[ "$DRY_RUN" == "true" ]]; then
        echo "  [DRY RUN] Would merge PR #$number ($head): $title"
        if [[ ${#REPO_FLAG[@]} -gt 0 ]]; then
            echo "  [DRY RUN]   gh pr merge $number ${REPO_FLAG[*]} --merge --delete-branch"
        else
            echo "  [DRY RUN]   gh pr merge $number --merge --delete-branch"
        fi
        MERGED_LIST+="#$number $title"$'\n'
        continue
    fi

    echo "  Merging PR #$number and deleting branch '$head'..."
    if gh pr merge "$number" "${REPO_FLAG[@]}" --merge --delete-branch; then
        echo "  Successfully merged and deleted branch."
        MERGED_LIST+="#$number $title"$'\n'
    else
        echo "  Error: Failed to merge PR #$number."
        MERGE_ERROR_LIST+="#$number $title"$'\n'
        MERGE_ERROR_OCCURRED=true
    fi
done <<< "$PR_LIST"

print_bucket() {
    local label="$1"
    local list="$2"
    local count=0
    if [[ -n "$list" ]]; then
        count=$(echo -n "$list" | grep -c '^#')
    fi
    echo "$label: $count"
    if [[ "$count" -gt 0 ]]; then
        echo "$list" | sed '/^$/d' | sed 's/^/    /'
    fi
}

echo
echo "===================================================="
echo "Summary"
echo "===================================================="
if [[ "$DRY_RUN" == "true" ]]; then
    print_bucket "Would merge" "$MERGED_LIST"
else
    print_bucket "Merged" "$MERGED_LIST"
fi
print_bucket "Failing checks" "$FAILED_CHECKS_LIST"
print_bucket "Checks pending" "$PENDING_CHECKS_LIST"
print_bucket "Merge conflict" "$CONFLICT_LIST"
print_bucket "Mergeability unknown" "$UNKNOWN_MERGEABLE_LIST"
print_bucket "Merge command failed" "$MERGE_ERROR_LIST"

if [[ "$MERGE_ERROR_OCCURRED" == "true" ]]; then
    exit 1
fi

exit 0
