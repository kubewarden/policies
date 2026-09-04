package main

import (
	kubewarden "github.com/kubewarden/policy-sdk-go"
	wapc "github.com/wapc/wapc-guest-tinygo"
)

func main() {
	wapc.RegisterFunctions(wapc.Functions{
		"validate":          validate,
		"validate_settings": func(_ []byte) ([]byte, error) { return kubewarden.AcceptSettings() },
	})
}
