package main

import (
	"flag"
	"os"
	"text/template"
)

const fileTemplate = `_extends: policies:release-drafter.yml
name-template: "{{.PolicyName}}/v$RESOLVED_VERSION"
tag-template: "{{.PolicyName}}/v$RESOLVED_VERSION"
tag-prefix: {{.PolicyName}}/v
include-paths:
  - "policies/{{.PolicyName}}/"
`

func main() {
	// parse command line argument to get policy name and directory
	var policyName string
	var outFile string
	flag.StringVar(&policyName, "policy-name", "", "The name of the policy to be used in the tag template")
	flag.StringVar(&outFile, "output", "", "The output file to write the generated config to")

	flag.Parse()

	if policyName == "" || outFile == "" {
		panic("Both --policy-name and --output must be provided")
	}

	if _, err := os.Stat("policies/" + policyName); err != nil {
		panic(err)
	}

	template := template.Must(template.New("release-drafter").Parse(fileTemplate))
	// open output file
	out, err := os.Create(outFile)
	if err != nil {
		panic(err)
	}
	template.Execute(out, map[string]string{
		"PolicyName": policyName,
	})
}
