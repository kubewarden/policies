package main

import (
	"errors"
	"strings"
	"testing"
)

func TestCheckMalformedQuantities(t *testing.T) {
	cases := []struct {
		desc                       string
		nsLimit, prjLimit, prjUsed string
		expectedString             string
	}{
		{"Bad namespace", "boom", "1k", "2k", "namespace"},
		{"Bad project limit", "1k", "boom", "2k", "project limit"},
		{"Bad project used quota", "1k", "2k", "boom", "project used quota"},
	}

	for _, tc := range cases {
		err := checkLimitVsAvailableQuota(tc.nsLimit, tc.prjLimit, tc.prjUsed)

		var parseErr *QuantityParseError
		switch {
		case err == nil:
			t.Errorf("%s: should have raised an error", tc.desc)
		case errors.As(err, &parseErr):
			if !strings.Contains(parseErr.Error(), tc.expectedString) {
				t.Errorf("%s: the error was supposed to be about '%s': %v", tc.desc, tc.expectedString, parseErr)
			}
		default:
			t.Errorf("%s: didn't get the expected error: %v", tc.desc, err)
		}
	}
}

func TestNamespaceRequestExceedsAvailability(t *testing.T) {
	cases := []struct {
		desc                       string
		nsLimit, prjLimit, prjUsed string
	}{
		{"Way above", "2k", "1k", "500"},
		{"Above", "500", "1k", "1k"},
		{"Already overcommited", "500", "1k", "2k"},
	}

	for _, tc := range cases {
		err := checkLimitVsAvailableQuota(tc.nsLimit, tc.prjLimit, tc.prjUsed)
		var exceedsErr *NamespaceRequestExceedsAvailabilityError
		switch {
		case err == nil:
			t.Errorf("%s: should have raised an error", tc.desc)
		case errors.As(err, &exceedsErr):
		default:
			t.Errorf("%s: didn't get the expected error: %v", tc.desc, err)
		}
	}
}

func TestLimitIsReasonable(t *testing.T) {
	cases := []struct {
		desc                       string
		nsLimit, prjLimit, prjUsed string
	}{
		{"Equal", "1k", "2k", "1k"},
		{"Below", "1k", "1M", "2k"},
		{"No usage", "1k", "1M", "0"},
		{"Not interested", "0", "1M", "1M"},
	}

	for _, tc := range cases {
		err := checkLimitVsAvailableQuota(tc.nsLimit, tc.prjLimit, tc.prjUsed)
		if err != nil {
			t.Errorf("%s: should not have raised an error: %v", tc.desc, err)
		}
	}
}

func TestValidateQuotas(t *testing.T) {
	cases := []struct {
		desc        string
		project     *Project
		nsLimits    *ResourceQuotaLimit
		expectError bool
	}{
		{
			"Project.Spec.ResourceQuota is nil",
			&Project{Spec: &ProjectSpec{ResourceQuota: nil}},
			nil,
			false,
		},
		{
			"Project.Spec.ResourceQuota has nothing set",
			&Project{Spec: &ProjectSpec{ResourceQuota: &ProjectResourceQuota{}}},
			nil,
			false,
		},
		{
			"nsLimits is nil",
			&Project{
				Spec: &ProjectSpec{
					ResourceQuota: &ProjectResourceQuota{
						Limit: ResourceQuotaLimit{
							Pods: "100",
						},
						UsedLimit: ResourceQuotaLimit{
							Pods: "2",
						},
					},
				},
			},
			nil,
			false,
		},
		{
			"a legitimate request",
			&Project{
				Spec: &ProjectSpec{
					ResourceQuota: &ProjectResourceQuota{
						Limit: ResourceQuotaLimit{
							Pods:     "100",
							Services: "100",
						},
						UsedLimit: ResourceQuotaLimit{
							Pods:     "50",
							Services: "100",
						},
					},
				},
			},
			&ResourceQuotaLimit{
				Pods: "50",
			},
			false,
		},
		{
			"requesting something that is not allowed",
			&Project{
				Spec: &ProjectSpec{
					ResourceQuota: &ProjectResourceQuota{
						Limit: ResourceQuotaLimit{
							Pods:                  "100",
							ServicesLoadBalancers: "0",
						},
						UsedLimit: ResourceQuotaLimit{
							Pods:                  "50",
							ServicesLoadBalancers: "0",
						},
					},
				},
			},
			&ResourceQuotaLimit{
				ServicesLoadBalancers: "1",
			},
			true,
		},
	}

	for _, tc := range cases {
		err := validateQuotas(tc.project, tc.nsLimits)

		if !tc.expectError && err != nil {
			t.Errorf("%s: got an unexpected error: %v", tc.desc, err)
		}
		if tc.expectError && err == nil {
			t.Errorf("%s: was expecting an error", tc.desc)
		}
	}
}
