package main

import (
	kubewarden "github.com/kubewarden/policy-sdk-go"
)

func validateSettings(_ []byte) ([]byte, error) {
	return kubewarden.AcceptSettings()
}
