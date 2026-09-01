package main

import (
	"io"
	"log"
	"os"
)

func main() {
	const expectedArgsCount = 2

	if len(os.Args) != expectedArgsCount {
		log.Fatalln("Wrong usage, expected either 'validate' or `validate-settings'")
	}

	input, err := io.ReadAll(os.Stdin)
	if err != nil {
		log.Panicf("Cannot read input: %v", err)
	}

	var response []byte

	switch os.Args[1] {
	case "validate":
		response, err = validate(input)
	case "validate-settings":
		response, err = validateSettings(input)
	default:
		//nolint:gosec // os.Args[1] is the subcommand set by the host, not external untrusted input
		log.Fatalf("wrong subcommand: '%s' - use either 'validate' or 'validate-settings'", os.Args[1])
	}

	if err != nil {
		log.Fatal(err)
	}

	_, err = os.Stdout.Write(response)
	if err != nil {
		log.Fatalf("Cannot write response: %v", err)
	}
}
