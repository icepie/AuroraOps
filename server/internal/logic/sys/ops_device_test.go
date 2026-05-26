package sys

import "testing"

func TestNormalizeDeviceArchitectureMigratesLegacyLocation(t *testing.T) {
	if got := normalizeDeviceArchitecture("", "aarch64"); got != "aarch64" {
		t.Fatalf("normalizeDeviceArchitecture legacy location = %q, want aarch64", got)
	}
	if got := normalizeDeviceArchitecture("x86_64", "机房A"); got != "x86_64" {
		t.Fatalf("normalizeDeviceArchitecture explicit value = %q, want x86_64", got)
	}
	if got := normalizeDeviceArchitecture("", "机房A"); got != "" {
		t.Fatalf("normalizeDeviceArchitecture location text = %q, want empty", got)
	}
}

func TestNormalizeDeviceLocationHidesArchitectureValues(t *testing.T) {
	if got := normalizeDeviceLocation("x86_64"); got != "" {
		t.Fatalf("normalizeDeviceLocation arch = %q, want empty", got)
	}
	if got := normalizeDeviceLocation("上海机房"); got != "上海机房" {
		t.Fatalf("normalizeDeviceLocation text = %q, want 上海机房", got)
	}
}
