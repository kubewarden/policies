package main

//nolint:godox // tracked TODO, not a lint violation to fix now
// TODO: figure out if it's worth to move this to a dedicated library

// Message is the optional string used to build validation responses.
type Message string

// Code is the optional error code associated with validation responses.
type Code uint16

const (
	// NoMessage can be used when building a response that doesn't have any
	// message to be shown to the user.
	NoMessage Message = ""

	// NoCode can be used when building a response that doesn't have any
	// error code to be shown to the user.
	NoCode Code = 0
)

// ValidationResponse defines the response given when validating a request.
type ValidationResponse struct {
	Accepted bool `json:"accepted"`
	// Optional - ignored if accepted
	Message *string `json:"message,omitempty"`
	// Optional - ignored if accepted
	Code *uint16 `json:"code,omitempty"`
}

// SettingsValidationResponse is the response sent by a policy when validating
// its settings.
type SettingsValidationResponse struct {
	Valid bool `json:"valid"`
	// Optional - ignored if valid
	Message *string `json:"message,omitempty"`
}

// AcceptRequest can be used inside of the `validate` function to accept the
// incoming request.
func AcceptRequest() ValidationResponse {
	return ValidationResponse{
		Accepted: true,
	}
}

// RejectRequest can be used inside of the `validate` function to reject the
// incoming request
// * `message`: optional message to show to the user
// * `code`: optional error code to show to the user.
func RejectRequest(message Message, code Code) ValidationResponse {
	response := ValidationResponse{
		Accepted: false,
	}

	if message != NoMessage {
		msg := string(message)
		response.Message = &msg
	}

	if code != NoCode {
		c := uint16(code)
		response.Code = &c
	}

	return response
}

// AcceptSettings be used inside of the `validateSettings` function to accept the
// incoming settings.
func AcceptSettings() SettingsValidationResponse {
	return SettingsValidationResponse{
		Valid: true,
	}
}

// RejectSettings can be used inside of the `validate_settings` function to
// mark the user provided settings as invalid
// * `message`: optional message to show to the user.
func RejectSettings(message Message) SettingsValidationResponse {
	response := SettingsValidationResponse{
		Valid: false,
	}

	if message != NoMessage {
		msg := string(message)
		response.Message = &msg
	}

	return response
}
