package sys

import (
	"testing"

	"auroraops/internal/model/entity"
)

func TestNormalizeAssetTypeFastfetchAliases(t *testing.T) {
	cases := map[string]string{
		"network":           "network",
		"nic":               "network",
		"network_interface": "network",
		"gpu":               "gpu",
		"video":             "gpu",
		"graphics":          "gpu",
		"graphics_card":     "gpu",
		"disk":              "disk",
		"storage":           "disk",
		"physical_disk":     "disk",
		"physicaldisk":      "disk",
		"drive":             "disk",
	}
	for input, want := range cases {
		if got := normalizeAssetType(input); got != want {
			t.Fatalf("normalizeAssetType(%q) = %q, want %q", input, got, want)
		}
	}
}

func TestBuildOverviewRowIncludesDiskAndKeepsAssetOrder(t *testing.T) {
	s := &sSysOpsHardware{}
	row := s.buildOverviewRow(&entity.OpsDevice{Id: 1, Name: "node-1"}, "", []*entity.OpsAsset{
		{
			AssetType:     "storage",
			AssetName:     "NVMe Disk",
			Model:         "SN850X",
			Specification: "2 TB / NVMe / /dev/nvme0n1",
		},
		{
			AssetType:     "disk",
			AssetName:     "SATA Disk",
			Model:         "ST2000",
			Specification: "2 TB / SATA / /dev/sda",
		},
		{
			AssetType: "network_interface",
			AssetName: "eth0",
			SerialNo:  "aa:bb:cc:dd:ee:ff",
		},
		{
			AssetType: "graphics",
			AssetName: "RTX 4060",
		},
	})

	if row.GroupName != "未分组" {
		t.Fatalf("GroupName = %q, want 未分组", row.GroupName)
	}
	if row.Disk != "NVMe Disk / SN850X / 2 TB / NVMe / /dev/nvme0n1；SATA Disk / ST2000 / 2 TB / SATA / /dev/sda" {
		t.Fatalf("Disk = %q", row.Disk)
	}
	if row.Nic != "eth0" {
		t.Fatalf("Nic = %q, want eth0", row.Nic)
	}
	if row.Gpu != "RTX 4060" {
		t.Fatalf("Gpu = %q, want RTX 4060", row.Gpu)
	}
}
