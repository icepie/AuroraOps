package sys

import (
	"testing"

	"auroraops/internal/dao"
	"auroraops/internal/model/entity"
	"auroraops/internal/model/input/sysin"
)

func TestOpsAssetUniqueKeySelection(t *testing.T) {
	s := &sSysOpsAsset{}

	incoming := &sysin.OpsAssetSyncItem{
		AssetType: "disk",
		UniqueKey: "disk-by-path",
		SerialNo:  "disk-serial",
	}
	if got := s.assetIncomingUniqueKey(incoming); got != "disk-by-path" {
		t.Fatalf("assetIncomingUniqueKey should prefer uniqueKey, got %q", got)
	}

	incoming.UniqueKey = ""
	if got := s.assetIncomingUniqueKey(incoming); got != "disk-serial" {
		t.Fatalf("assetIncomingUniqueKey should fall back to serialNo, got %q", got)
	}

	stored := &entity.OpsAsset{
		AssetType: "disk",
		UniqueKey: "stored-unique",
		SerialNo:  "stored-serial",
	}
	if got := s.assetStoredUniqueKey(stored, opsAssetSchema{HasUniqueKey: true}); got != "stored-unique" {
		t.Fatalf("assetStoredUniqueKey should use unique_key when present, got %q", got)
	}

	stored.UniqueKey = ""
	if got := s.assetStoredUniqueKey(stored, opsAssetSchema{HasUniqueKey: true}); got != "stored-serial" {
		t.Fatalf("assetStoredUniqueKey should fall back to serial_no when unique_key is empty, got %q", got)
	}

	stored.UniqueKey = "ignored-on-old-schema"
	if got := s.assetStoredUniqueKey(stored, opsAssetSchema{}); got != "stored-serial" {
		t.Fatalf("assetStoredUniqueKey should use serial_no for old schema, got %q", got)
	}
}

func TestOpsAssetSyncFieldsRespectSchema(t *testing.T) {
	s := &sSysOpsAsset{}
	columns := dao.OpsAsset.Columns()

	oldSchemaFields := s.assetSyncInsertFields(opsAssetSchema{})
	for _, field := range []string{
		columns.UniqueKey,
		columns.Source,
		columns.SyncHash,
		columns.LastSeenAt,
	} {
		if hasAny(oldSchemaFields, field) {
			t.Fatalf("old schema fields should not include %q: %#v", field, oldSchemaFields)
		}
	}

	newSchemaFields := s.assetSyncInsertFields(opsAssetSchema{
		HasUniqueKey:  true,
		HasSource:     true,
		HasSyncHash:   true,
		HasLastSeenAt: true,
	})
	for _, field := range []string{
		columns.UniqueKey,
		columns.Source,
		columns.SyncHash,
		columns.LastSeenAt,
	} {
		if !hasAny(newSchemaFields, field) {
			t.Fatalf("new schema fields should include %q: %#v", field, newSchemaFields)
		}
	}
}

func TestOpsAssetSyncHashIncludesSerialNo(t *testing.T) {
	s := &sSysOpsAsset{}
	base := &sysin.OpsAssetSyncItem{
		AssetType:     "memory",
		UniqueKey:     "slot-a",
		AssetName:     "16 GB DDR4",
		Brand:         "Kingston",
		Model:         "KVR",
		SerialNo:      "serial-a",
		Specification: "16 GB / DDR4",
		Source:        "fastfetch-sys",
		Remark:        "auto:fastfetch-sys",
	}
	changed := *base
	changed.SerialNo = "serial-b"

	if got, wantNot := s.assetSyncHash(base), s.assetSyncHash(&changed); got == wantNot {
		t.Fatalf("assetSyncHash should change when serialNo changes, got %q", got)
	}
}

func TestOpsAssetAutoSyncedAssetDetection(t *testing.T) {
	s := &sSysOpsAsset{}
	if s.isAutoSyncedAsset(&entity.OpsAsset{Source: "manual"}) {
		t.Fatal("manual source should not be treated as auto-synced")
	}
	if s.isAutoSyncedAsset(&entity.OpsAsset{Source: ""}) {
		t.Fatal("empty source without auto remark should not be treated as auto-synced")
	}
	if !s.isAutoSyncedAsset(&entity.OpsAsset{Source: "fastfetch-sys"}) {
		t.Fatal("fastfetch-sys source should be treated as auto-synced")
	}
	if !s.isAutoSyncedAsset(&entity.OpsAsset{Remark: "auto:fastfetch-sys"}) {
		t.Fatal("auto remark should be treated as auto-synced for compatibility")
	}
}

func hasAny(values []any, want string) bool {
	for _, value := range values {
		if text, ok := value.(string); ok && text == want {
			return true
		}
	}
	return false
}
