// This program generates release-please-config.json and
// .release-please-manifest.json from the policies in the "policies"
// directory. Run it with `make release-please-config` after adding,
// removing, or renaming a policy.
package main

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"regexp"
	"sort"
)

const (
	configPath   = "release-please-config.json"
	manifestPath = ".release-please-manifest.json"
	policiesDir  = "policies"
)

var versionAnnotationRe = regexp.MustCompile(`(?m)^\s*io\.kubewarden\.policy\.version:\s*"?([0-9][^"\s]*)"?\s*(#.*)?$`)
var versionMarkerRe = regexp.MustCompile(`(?m)^\s*io\.kubewarden\.policy\.version:.*x-release-please-version\s*$`)
var releaseTagLineRe = regexp.MustCompile(`(?m)^\s*com\.github\.release\.tag:`)
var releaseTagMarkerRe = regexp.MustCompile(`(?m)^\s*com\.github\.release\.tag:.*x-release-please-version\s*$`)
var cargoPackageNameRe = regexp.MustCompile(`(?m)^\s*name\s*=\s*"([^"]+)"\s*$`)

type packageConfig struct {
	Component   string          `json:"component"`
	ReleaseType string          `json:"release-type"`
	ExtraFiles  []extraFileSpec `json:"extra-files"`
}

type extraFileSpec struct {
	Type     string `json:"type,omitempty"`
	Path     string `json:"path,omitempty"`
	JSONPath string `json:"jsonpath,omitempty"`
}

type config struct {
	Schema                  string                   `json:"$schema"`
	BootstrapSHA            string                   `json:"bootstrap-sha,omitempty"`
	SeparatePullRequests    bool                     `json:"separate-pull-requests"`
	SkipChangelog           bool                     `json:"skip-changelog"`
	Draft                   bool                     `json:"draft"`
	ForceTagCreation        bool                     `json:"force-tag-creation"`
	IncludeComponentInTag   bool                     `json:"include-component-in-tag"`
	IncludeVInTag           bool                     `json:"include-v-in-tag"`
	TagSeparator            string                   `json:"tag-separator"`
	ExtraLabel              string                   `json:"extra-label"`
	PullRequestTitlePattern string                   `json:"pull-request-title-pattern"`
	ReleaseSearchDepth      int                      `json:"release-search-depth"`
	SequentialCalls         bool                     `json:"sequential-calls"`
	Packages                map[string]packageConfig `json:"packages"`
}

func main() {
	var bootstrapSHA string
	for i, arg := range os.Args {
		if arg == "--bootstrap-sha" && i+1 < len(os.Args) {
			bootstrapSHA = os.Args[i+1]
		}
	}

	// Once release-please-config.json exists, keep its "bootstrap-sha"
	// unless a new value is given explicitly. release-please ignores this
	// key after the first release PR merges, but this generator must not
	// erase it on every re-run, or "make release-please-config" would
	// produce a diff on every CI run.
	if bootstrapSHA == "" {
		if existing, err := os.ReadFile(configPath); err == nil {
			var previous config
			if err := json.Unmarshal(existing, &previous); err == nil {
				bootstrapSHA = previous.BootstrapSHA
			}
		}
	}

	entries, err := os.ReadDir(policiesDir)
	if err != nil {
		panic(err)
	}

	packages := map[string]packageConfig{}
	manifest := map[string]string{}

	for _, entry := range entries {
		if !entry.IsDir() {
			continue
		}
		name := entry.Name()
		policyDir := filepath.Join(policiesDir, name)
		metadataPath := filepath.Join(policyDir, "metadata.yml")
		metadataBytes, err := os.ReadFile(metadataPath)
		if err != nil {
			// Not a policy directory (e.g. "crates").
			continue
		}

		releaseType := "simple"
		cargoTomlPath := filepath.Join(policyDir, "Cargo.toml")
		var crateName string
		if cargoBytes, err := os.ReadFile(cargoTomlPath); err == nil {
			releaseType = "rust"
			match := cargoPackageNameRe.FindSubmatch(cargoBytes)
			if match == nil {
				panic(fmt.Sprintf("%s: could not find [package] name", cargoTomlPath))
			}
			crateName = string(match[1])
		}

		// The "generic" updater is the only one usable on metadata.yml.
		// The "yaml" updater fully re-serializes the document, which
		// strips every comment (including the marker the "generic"
		// updater itself relies on) and reformats the file. So every
		// version annotation in metadata.yml must carry the
		// "x-release-please-version" marker instead; this generator
		// refuses to run otherwise, since a missing marker would make
		// release-please silently skip that annotation on a release.
		if !versionMarkerRe.Match(metadataBytes) {
			panic(fmt.Sprintf("%s: io.kubewarden.policy.version is missing the x-release-please-version marker", metadataPath))
		}
		if releaseTagLineRe.Match(metadataBytes) && !releaseTagMarkerRe.Match(metadataBytes) {
			panic(fmt.Sprintf("%s: com.github.release.tag is missing the x-release-please-version marker", metadataPath))
		}

		key := policiesDir + "/" + name
		extraFiles := []extraFileSpec{
			{
				Type: "generic",
				Path: "metadata.yml",
			},
		}

		if releaseType == "rust" {
			// A Rust policy needs a third entry, to update its version in
			// the shared lock file policies/Cargo.lock.
			//
			// release-please's "cargo-workspace" plugin would normally do
			// this, and its documentation recommends it for Rust monorepos.
			// It cannot be used here: the plugin reads the workspace
			// manifest from the repository root
			// (src/plugins/cargo-workspace.ts in release-please), and this
			// repository keeps it at policies/Cargo.toml instead. The
			// plugin has no option to point it elsewhere, and fails with
			// "Failed to find file: Cargo.toml" when it looks at the root.
			//
			// Without the plugin, the "rust" strategy still bumps
			// policies/<policy>/Cargo.toml correctly, since each policy is
			// released as its own crate. But it looks for that crate's
			// lock file at policies/<policy>/Cargo.lock, which does not
			// exist; the real one, policies/Cargo.lock, is never touched.
			// A release PR that leaves it behind would fail CI, because
			// policies/Makefile.rust builds every Rust policy with
			// --locked.
			//
			// So this entry is written by hand instead of through the
			// plugin. The leading slash makes the path relative to the
			// repository root, not to this policy's directory (see
			// BaseStrategy.addPath in release-please). The jsonpath must
			// match on the crate name from [package], not the directory
			// name: several Rust policies use a different one, for example
			// volumeMounts-policy is the crate volumemounts-policy. Keep
			// this entry in sync by running `make release-please-config`
			// again after renaming a policy directory or a crate.
			//
			// The filter compares against "@.name.value", not "@.name".
			// release-please's TOML updater parses with a "tagged" parser
			// (src/util/toml-edit.ts) that wraps every scalar as
			// {start, end, value}, so it can splice the replacement back
			// into the original text without reformatting the file. A
			// filter written the ordinary way, "@.name=='...'", compares
			// that wrapper object to a string and never matches; verified
			// empirically against release-please's own GenericToml class
			// and this repository's actual policies/Cargo.lock. This is
			// an internal detail of release-please's TOML updater, not
			// documented, and could change in a future release; a
			// regression would fail loudly, as a release pull request
			// whose build fails under `cargo --locked`.
			extraFiles = append(extraFiles, extraFileSpec{
				Type:     "toml",
				Path:     "/policies/Cargo.lock",
				JSONPath: fmt.Sprintf("$.package[?(@.name.value=='%s')].version", crateName),
			})
		}

		packages[key] = packageConfig{
			Component:   name,
			ReleaseType: releaseType,
			ExtraFiles:  extraFiles,
		}

		match := versionAnnotationRe.FindSubmatch(metadataBytes)
		if match == nil {
			panic(fmt.Sprintf("%s: could not find io.kubewarden.policy.version annotation", metadataPath))
		}
		manifest[key] = string(match[1])
	}

	// No "plugins" key is set here on purpose. Do not add the
	// "cargo-workspace" plugin back without reading the comment above the
	// per-Rust-policy extra-files entry: this repository's Cargo workspace
	// lives at policies/Cargo.toml, not at the repository root, and that
	// plugin cannot be told to look there.
	cfg := config{
		Schema:               "https://raw.githubusercontent.com/googleapis/release-please/main/schemas/config.json",
		BootstrapSHA:         bootstrapSHA,
		SeparatePullRequests: true,
		SkipChangelog:        true,
		// The release stays a draft until release.yml attaches the wasm
		// module and the SBOM to it, then publishes it.
		Draft: true,
		// Required alongside "draft". GitHub does not create the git tag
		// for a draft release until it is published, so without this,
		// release-please would fail to find the previous release on its
		// next run.
		ForceTagCreation:      true,
		IncludeComponentInTag: true,
		IncludeVInTag:         true,
		TagSeparator:          "/",
		ExtraLabel:            "kind/chore,area/release",
		// PullRequestTitle.toString() already inserts a space before the
		// component when ${component} is not empty, so the pattern must
		// not add a second one. Verified: with a leading space here, the
		// title read "build: Prepare for release  <policy> <version>",
		// with two spaces.
		PullRequestTitlePattern: "build: Prepare for release${component} ${version}",
		ReleaseSearchDepth:      600,
		SequentialCalls:         true,
		Packages:                packages,
	}

	writeJSON(configPath, cfg)
	writeJSON(manifestPath, sortedManifest(manifest))
}

// sortedManifest returns the manifest map unchanged; Go's json encoder
// sorts map keys alphabetically already, this helper only documents intent.
func sortedManifest(m map[string]string) map[string]string {
	keys := make([]string, 0, len(m))
	for k := range m {
		keys = append(keys, k)
	}
	sort.Strings(keys)
	return m
}

func writeJSON(path string, v interface{}) {
	data, err := json.MarshalIndent(v, "", "  ")
	if err != nil {
		panic(err)
	}
	data = append(data, '\n')
	if err := os.WriteFile(path, data, 0o644); err != nil {
		panic(err)
	}
}
