package sys

import (
	"bytes"
	"testing"
)

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

func TestParseWakeMacAcceptsCommonFormats(t *testing.T) {
	cases := []string{
		"00:11:22:33:44:55",
		"00-11-22-33-44-55",
		"0011.2233.4455",
		"001122334455",
	}

	for _, tc := range cases {
		mac, err := parseWakeMac(tc)
		if err != nil {
			t.Fatalf("parseWakeMac(%q) unexpected error: %v", tc, err)
		}
		if got := formatWakeMac(mac); got != "00:11:22:33:44:55" {
			t.Fatalf("parseWakeMac(%q) = %q, want 00:11:22:33:44:55", tc, got)
		}
	}
}

func TestBuildMagicPacket(t *testing.T) {
	mac, err := parseWakeMac("00:11:22:33:44:55")
	if err != nil {
		t.Fatal(err)
	}

	packet := buildMagicPacket(mac)
	if len(packet) != 102 {
		t.Fatalf("magic packet length = %d, want 102", len(packet))
	}
	if !bytes.Equal(packet[:6], []byte{0xff, 0xff, 0xff, 0xff, 0xff, 0xff}) {
		t.Fatalf("magic packet prefix = %v, want ff x 6", packet[:6])
	}
	for i := 0; i < 16; i++ {
		chunk := packet[6+i*6 : 6+(i+1)*6]
		if !bytes.Equal(chunk, mac[:]) {
			t.Fatalf("magic packet mac chunk %d = %v, want %v", i, chunk, mac)
		}
	}
}

func TestBuildWakeTargets(t *testing.T) {
	got := buildWakeTargets("192.168.10.255", "192.168.10.31", 9)
	want := []string{"192.168.10.255:9", "255.255.255.255:9"}
	if len(got) != len(want) {
		t.Fatalf("targets = %#v, want %#v", got, want)
	}
	for i := range want {
		if got[i] != want[i] {
			t.Fatalf("targets[%d] = %q, want %q", i, got[i], want[i])
		}
	}

	got = buildWakeTargets("", "10.20.30.40", 7)
	want = []string{"10.20.30.255:7", "255.255.255.255:7"}
	if len(got) != len(want) {
		t.Fatalf("targets = %#v, want %#v", got, want)
	}
	for i := range want {
		if got[i] != want[i] {
			t.Fatalf("targets[%d] = %q, want %q", i, got[i], want[i])
		}
	}
}
