package main

type Request struct {
	User     string `json:"user"`
	Action   string `json:"action"`
	Resource string `json:"resource"`
}

// RawValidationRequest describes the input received by the policy
// when invoked via the `validate` subcommand.
type RawValidationRequest struct {
	Request  Request  `json:"request"`
	Settings Settings `json:"settings"`
}
