module github.com/kubewarden/container-resources-policy

go 1.24.0

toolchain go1.25.7

require (
	github.com/francoispqt/onelog v0.0.0-20190306043706-8c2bb31b10a4
	github.com/google/go-cmp v0.7.0
	github.com/google/gofuzz v1.2.0
	github.com/kubewarden/k8s-objects v1.29.0-kw1
	github.com/kubewarden/policy-sdk-go v0.12.0
	github.com/spf13/pflag v1.0.10
	github.com/stretchr/testify v1.11.1
	github.com/wapc/wapc-guest-tinygo v0.3.3
	gopkg.in/inf.v0 v0.9.1
)

require (
	github.com/davecgh/go-spew v1.1.1 // indirect
	github.com/francoispqt/gojay v0.0.0-20181220093123-f2cc13a668ca // indirect
	github.com/go-openapi/strfmt v0.21.3 // indirect
	github.com/pmezard/go-difflib v1.0.0 // indirect
	gopkg.in/yaml.v3 v3.0.1 // indirect
)

replace github.com/go-openapi/strfmt => github.com/kubewarden/strfmt v0.1.3
