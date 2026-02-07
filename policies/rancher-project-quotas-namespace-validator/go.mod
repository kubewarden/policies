module github.com/kubewarden/rancher-project-quotas-namespace-validator

go 1.22

toolchain go1.25.7

replace github.com/go-openapi/strfmt => github.com/kubewarden/strfmt v0.1.3

require (
	github.com/kubewarden/k8s-objects v1.29.0-kw1
	github.com/kubewarden/policy-sdk-go v0.12.0
	github.com/wapc/wapc-guest-tinygo v0.3.3
	gopkg.in/inf.v0 v0.9.1
)

require (
	github.com/davecgh/go-spew v1.1.1 // indirect
	github.com/go-openapi/strfmt v0.21.5 // indirect
	github.com/pmezard/go-difflib v1.0.0 // indirect
	github.com/stretchr/objx v0.5.2 // indirect
	github.com/stretchr/testify v1.10.0 // indirect
	gopkg.in/yaml.v3 v3.0.1 // indirect
)
