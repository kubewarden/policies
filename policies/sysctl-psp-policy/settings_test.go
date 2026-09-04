package main

import (
	"testing"
)

func TestParsingSettingsWithAllValuesProvidedFromValidationReq(t *testing.T) {
	request := `
	{
		"request": "doesn't matter here",
		"settings": {
			"allowedUnsafeSysctls": ["net.core.somaxconn"],
			"forbiddenSysctls": ["kernel.shm_rmid_forced", "net.*"]
		}
	}
	`
	rawRequest := []byte(request)

	settings, err := NewSettingsFromValidationReq(rawRequest)
	if err != nil {
		t.Errorf("Unexpected error %+v", err)
	}

	expected := []string{"net.core.somaxconn"}
	for _, exp := range expected {
		if !settings.AllowedUnsafeSysctls.Contains(exp) {
			t.Errorf("Missing value %s", exp)
		}
	}

	expected = []string{"kernel.shm_rmid_forced", "net.*"}
	for _, exp := range expected {
		if !settings.ForbiddenSysctls.Contains(exp) {
			t.Errorf("Missing value %s", exp)
		}
	}
}

func TestParsingSettingsWithNoValueProvided(t *testing.T) {
	request := `
	{
		"request": "doesn't matter here",
		"settings": {
		}
	}
	`
	rawRequest := []byte(request)

	settings, err := NewSettingsFromValidationReq(rawRequest)
	if err != nil {
		t.Errorf("Unexpected error %+v", err)
	}

	if settings.AllowedUnsafeSysctls.Cardinality() != 0 {
		t.Errorf("Expected AllowedUnsafeSysctls to be empty")
	}

	if settings.ForbiddenSysctls.Cardinality() != 0 {
		t.Errorf("Expected ForbiddenSysctls to be empty")
	}
}

func TestSettingsAreValid(t *testing.T) {
	for _, tcase := range []struct {
		name      string
		request   string
		wantError bool
		error     string
	}{
		{
			name: "empty settings",
			request: `
			{
				"request": "doesn't matter here",
				"settings": {
				}
			}
			`,
		},
		{
			name: "correct settings",
			request: `
			{
				"request": "doesn't matter here",
				"settings": {
					"allowedUnsafeSysctls": ["net.core.somaxconn"],
					"forbiddenSysctls": ["kernel.shm_rmid_forced", "net.*"]
				}
			}
			`,
		},
		{
			name: "allowedUnsafeSysctls doesn't accept patterns with *",
			request: `
			{
				"request": "doesn't matter here",
				"settings": {
					"allowedUnsafeSysctls": ["net.*"],
					"forbiddenSysctls": ["net.core.somaxconn"]
				}
			}
			`,
			wantError: true,
			error:     "allowedUnsafeSysctls doesn't accept patterns with `*`",
		},
		{
			name: "globs need to be suffix",
			request: `
			{
				"request": "doesn't matter here",
				"settings": {
					"allowedUnsafeSysctls": ["net.core.somaxconn"],
					"forbiddenSysctls": ["kernel.shm_rmid_forced", "net.*.foo"]
				}
			}
			`,
			wantError: true,
			error:     "forbiddenSysctls only accepts patterns with `*` as suffix",
		},
		{
			name: "sysctl in both fields",
			request: `
			{
				"request": "doesn't matter here",
				"settings": {
					"allowedUnsafeSysctls": ["net.core.somaxconn"],
					"forbiddenSysctls": ["net.core.somaxconn"]
				}
			}
			`,
			wantError: true,
			error:     "these sysctls cannot be allowed and forbidden at the same time: net.core.somaxconn",
		},
	} {
		t.Run(tcase.name, func(t *testing.T) {
			rawRequest := []byte(tcase.request)

			settings, _ := NewSettingsFromValidationReq(rawRequest)
			valid, err := settings.Valid()

			if err != nil {
				if !tcase.wantError {
					t.Errorf("on test %q, got unexpected error '%v'", tcase.name, err)
				} else if tcase.error != err.Error() {
					t.Errorf("on test %q, got error '%v', wanted error '%v'", tcase.name, err, tcase.error)
				}
			}
			if tcase.wantError && valid {
				t.Errorf("on test %q, settings are valid and we got unexpected error '%v'",
					tcase.name, err)
			}
		})
	}
}
