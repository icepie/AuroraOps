SET NAMES utf8mb4;
SET FOREIGN_KEY_CHECKS = 0;

START TRANSACTION;

CREATE TABLE IF NOT EXISTS `hg_ops_device_group` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT COMMENT '分组ID',
  `name` varchar(128) NOT NULL DEFAULT '' COMMENT '分组名称',
  `sort` int NOT NULL DEFAULT '0' COMMENT '排序',
  `remark` varchar(500) NOT NULL DEFAULT '' COMMENT '备注',
  `status` tinyint NOT NULL DEFAULT '1' COMMENT '状态，1正常，2停用',
  `created_at` datetime DEFAULT NULL COMMENT '创建时间',
  `updated_at` datetime DEFAULT NULL COMMENT '更新时间',
  `deleted_at` datetime DEFAULT NULL COMMENT '删除时间',
  PRIMARY KEY (`id`),
  KEY `idx_ops_device_group_name` (`name`),
  KEY `idx_ops_device_group_sort` (`sort`),
  KEY `idx_ops_device_group_status` (`status`),
  KEY `idx_ops_device_group_deleted_at` (`deleted_at`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci COMMENT='设备分组';

CREATE TABLE IF NOT EXISTS `hg_ops_device` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT COMMENT '设备ID',
  `group_id` bigint unsigned NOT NULL DEFAULT '0' COMMENT '分组ID',
  `name` varchar(128) NOT NULL DEFAULT '' COMMENT '设备名称',
  `hostname` varchar(128) NOT NULL DEFAULT '' COMMENT '主机名',
  `ip` varchar(64) NOT NULL DEFAULT '' COMMENT 'IP地址',
  `mac_address` varchar(32) NOT NULL DEFAULT '' COMMENT 'MAC地址',
  `device_type` varchar(64) NOT NULL DEFAULT '' COMMENT '设备类型',
  `os_name` varchar(128) NOT NULL DEFAULT '' COMMENT '操作系统',
  `architecture` varchar(64) NOT NULL DEFAULT '' COMMENT '系统架构',
  `location` varchar(255) NOT NULL DEFAULT '' COMMENT '部署位置',
  `monitor_snapshot` json DEFAULT NULL COMMENT '监视快照',
  `monitor_reported_at` datetime DEFAULT NULL COMMENT '监视上报时间',
  `sort` int NOT NULL DEFAULT '0' COMMENT '排序',
  `remark` varchar(500) NOT NULL DEFAULT '' COMMENT '备注',
  `status` tinyint NOT NULL DEFAULT '1' COMMENT '状态，1正常，2停用',
  `created_at` datetime DEFAULT NULL COMMENT '创建时间',
  `updated_at` datetime DEFAULT NULL COMMENT '更新时间',
  `deleted_at` datetime DEFAULT NULL COMMENT '删除时间',
  PRIMARY KEY (`id`),
  KEY `idx_ops_device_group_id` (`group_id`),
  KEY `idx_ops_device_name` (`name`),
  KEY `idx_ops_device_hostname` (`hostname`),
  KEY `idx_ops_device_ip` (`ip`),
  KEY `idx_ops_device_mac_address` (`mac_address`),
  KEY `idx_ops_device_status` (`status`),
  KEY `idx_ops_device_sort` (`sort`),
  KEY `idx_ops_device_deleted_at` (`deleted_at`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci COMMENT='运维设备';

SET @stmt := (
  SELECT IF(
    COUNT(*) = 0,
    'ALTER TABLE `hg_ops_device` ADD COLUMN `group_id` bigint unsigned NOT NULL DEFAULT ''0'' COMMENT ''分组ID'' AFTER `id`',
    'SELECT 1'
  )
  FROM information_schema.COLUMNS
  WHERE TABLE_SCHEMA = DATABASE()
    AND TABLE_NAME = 'hg_ops_device'
    AND COLUMN_NAME = 'group_id'
);
PREPARE stmt FROM @stmt;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

SET @stmt := (
  SELECT IF(
    COUNT(*) = 0,
    'ALTER TABLE `hg_ops_device` ADD COLUMN `mac_address` varchar(32) NOT NULL DEFAULT '''' COMMENT ''MAC地址'' AFTER `ip`',
    'SELECT 1'
  )
  FROM information_schema.COLUMNS
  WHERE TABLE_SCHEMA = DATABASE()
    AND TABLE_NAME = 'hg_ops_device'
    AND COLUMN_NAME = 'mac_address'
);
PREPARE stmt FROM @stmt;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

SET @stmt := (
  SELECT IF(
    COUNT(*) = 0,
    'CREATE INDEX `idx_ops_device_mac_address` ON `hg_ops_device` (`mac_address`)',
    'SELECT 1'
  )
  FROM information_schema.STATISTICS
  WHERE TABLE_SCHEMA = DATABASE()
    AND TABLE_NAME = 'hg_ops_device'
    AND INDEX_NAME = 'idx_ops_device_mac_address'
);
PREPARE stmt FROM @stmt;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

SET @stmt := (
  SELECT IF(
    COUNT(*) = 0,
    'CREATE INDEX `idx_ops_device_group_id` ON `hg_ops_device` (`group_id`)',
    'SELECT 1'
  )
  FROM information_schema.STATISTICS
  WHERE TABLE_SCHEMA = DATABASE()
    AND TABLE_NAME = 'hg_ops_device'
    AND INDEX_NAME = 'idx_ops_device_group_id'
);
PREPARE stmt FROM @stmt;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

SET @stmt := (
  SELECT IF(
    COUNT(*) = 0,
    'ALTER TABLE `hg_ops_device` ADD COLUMN `architecture` varchar(64) NOT NULL DEFAULT '''' COMMENT ''系统架构'' AFTER `os_name`',
    'SELECT 1'
  )
  FROM information_schema.COLUMNS
  WHERE TABLE_SCHEMA = DATABASE()
    AND TABLE_NAME = 'hg_ops_device'
    AND COLUMN_NAME = 'architecture'
);
PREPARE stmt FROM @stmt;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

SET @stmt := (
  SELECT IF(
    COUNT(*) = 0,
    'ALTER TABLE `hg_ops_device` ADD COLUMN `monitor_snapshot` json DEFAULT NULL COMMENT ''监视快照'' AFTER `location`',
    'SELECT 1'
  )
  FROM information_schema.COLUMNS
  WHERE TABLE_SCHEMA = DATABASE()
    AND TABLE_NAME = 'hg_ops_device'
    AND COLUMN_NAME = 'monitor_snapshot'
);
PREPARE stmt FROM @stmt;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

SET @stmt := (
  SELECT IF(
    COUNT(*) = 0,
    'ALTER TABLE `hg_ops_device` ADD COLUMN `monitor_reported_at` datetime DEFAULT NULL COMMENT ''监视上报时间'' AFTER `monitor_snapshot`',
    'SELECT 1'
  )
  FROM information_schema.COLUMNS
  WHERE TABLE_SCHEMA = DATABASE()
    AND TABLE_NAME = 'hg_ops_device'
    AND COLUMN_NAME = 'monitor_reported_at'
);
PREPARE stmt FROM @stmt;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

UPDATE `hg_ops_device`
SET `architecture` = `location`,
    `location` = ''
WHERE `architecture` = ''
  AND LOWER(`location`) IN ('aarch64', 'arm64', 'amd64', 'x86_64', 'i386', 'i686', 'loongarch64', 'mips64', 'mips64el', 'sw_64', 'riscv64');

CREATE TABLE IF NOT EXISTS `hg_ops_asset` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT COMMENT '资产ID',
  `device_id` bigint unsigned NOT NULL DEFAULT '0' COMMENT '所属设备ID',
  `asset_type` varchar(64) NOT NULL DEFAULT '' COMMENT '资产类型',
  `unique_key` varchar(191) NOT NULL DEFAULT '' COMMENT '资产唯一键',
  `asset_name` varchar(128) NOT NULL DEFAULT '' COMMENT '资产名称',
  `brand` varchar(128) NOT NULL DEFAULT '' COMMENT '品牌',
  `model` varchar(128) NOT NULL DEFAULT '' COMMENT '型号',
  `serial_no` varchar(128) NOT NULL DEFAULT '' COMMENT '序列号',
  `specification` varchar(500) NOT NULL DEFAULT '' COMMENT '规格参数',
  `source` varchar(32) NOT NULL DEFAULT 'manual' COMMENT '资产来源',
  `sync_hash` varchar(64) NOT NULL DEFAULT '' COMMENT '同步摘要',
  `last_seen_at` datetime DEFAULT NULL COMMENT '最近观测时间',
  `sort` int NOT NULL DEFAULT '0' COMMENT '排序',
  `remark` varchar(500) NOT NULL DEFAULT '' COMMENT '备注',
  `status` tinyint NOT NULL DEFAULT '1' COMMENT '状态，1正常，2停用',
  `created_at` datetime DEFAULT NULL COMMENT '创建时间',
  `updated_at` datetime DEFAULT NULL COMMENT '更新时间',
  `deleted_at` datetime DEFAULT NULL COMMENT '删除时间',
  PRIMARY KEY (`id`),
  KEY `idx_ops_asset_device_id` (`device_id`),
  KEY `idx_ops_asset_type` (`asset_type`),
  KEY `idx_ops_asset_device_type_unique` (`device_id`,`asset_type`,`unique_key`),
  KEY `idx_ops_asset_source` (`source`),
  KEY `idx_ops_asset_name` (`asset_name`),
  KEY `idx_ops_asset_status` (`status`),
  KEY `idx_ops_asset_sort` (`sort`),
  KEY `idx_ops_asset_deleted_at` (`deleted_at`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci COMMENT='运维资产';

SET @stmt := (
  SELECT IF(
    COUNT(*) = 0,
    'ALTER TABLE `hg_ops_asset` ADD COLUMN `unique_key` varchar(191) NOT NULL DEFAULT '''' COMMENT ''资产唯一键'' AFTER `asset_type`',
    'SELECT 1'
  )
  FROM information_schema.COLUMNS
  WHERE TABLE_SCHEMA = DATABASE()
    AND TABLE_NAME = 'hg_ops_asset'
    AND COLUMN_NAME = 'unique_key'
);
PREPARE stmt FROM @stmt;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

UPDATE `hg_ops_asset`
SET `unique_key` = `serial_no`
WHERE `unique_key` = ''
  AND `serial_no` <> '';

SET @stmt := (
  SELECT IF(
    COUNT(*) = 0,
    'CREATE INDEX `idx_ops_asset_device_type_unique` ON `hg_ops_asset` (`device_id`,`asset_type`,`unique_key`)',
    'SELECT 1'
  )
  FROM information_schema.STATISTICS
  WHERE TABLE_SCHEMA = DATABASE()
    AND TABLE_NAME = 'hg_ops_asset'
    AND INDEX_NAME = 'idx_ops_asset_device_type_unique'
);
PREPARE stmt FROM @stmt;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

SET @stmt := (
  SELECT IF(
    COUNT(*) = 0,
    'ALTER TABLE `hg_ops_asset` ADD COLUMN `source` varchar(32) NOT NULL DEFAULT ''manual'' COMMENT ''资产来源'' AFTER `specification`',
    'SELECT 1'
  )
  FROM information_schema.COLUMNS
  WHERE TABLE_SCHEMA = DATABASE()
    AND TABLE_NAME = 'hg_ops_asset'
    AND COLUMN_NAME = 'source'
);
PREPARE stmt FROM @stmt;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

SET @stmt := (
  SELECT IF(
    COUNT(*) = 0,
    'ALTER TABLE `hg_ops_asset` ADD COLUMN `sync_hash` varchar(64) NOT NULL DEFAULT '''' COMMENT ''同步摘要'' AFTER `source`',
    'SELECT 1'
  )
  FROM information_schema.COLUMNS
  WHERE TABLE_SCHEMA = DATABASE()
    AND TABLE_NAME = 'hg_ops_asset'
    AND COLUMN_NAME = 'sync_hash'
);
PREPARE stmt FROM @stmt;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

SET @stmt := (
  SELECT IF(
    COUNT(*) = 0,
    'ALTER TABLE `hg_ops_asset` ADD COLUMN `last_seen_at` datetime DEFAULT NULL COMMENT ''最近观测时间'' AFTER `sync_hash`',
    'SELECT 1'
  )
  FROM information_schema.COLUMNS
  WHERE TABLE_SCHEMA = DATABASE()
    AND TABLE_NAME = 'hg_ops_asset'
    AND COLUMN_NAME = 'last_seen_at'
);
PREPARE stmt FROM @stmt;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

SET @stmt := (
  SELECT IF(
    COUNT(*) = 0,
    'CREATE INDEX `idx_ops_asset_source` ON `hg_ops_asset` (`source`)',
    'SELECT 1'
  )
  FROM information_schema.STATISTICS
  WHERE TABLE_SCHEMA = DATABASE()
    AND TABLE_NAME = 'hg_ops_asset'
    AND INDEX_NAME = 'idx_ops_asset_source'
);
PREPARE stmt FROM @stmt;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

SET @now := NOW();

INSERT INTO `hg_admin_menu` (
  `pid`, `title`, `name`, `path`, `icon`, `type`, `redirect`, `permissions`,
  `permission_name`, `component`, `always_show`, `active_menu`, `is_root`, `is_frame`,
  `frame_src`, `keep_alive`, `hidden`, `affix`, `level`, `tree`, `sort`, `remark`,
  `status`, `created_at`, `updated_at`
)
SELECT
  0, '运维管理', 'opsManage', '/ops', 'ClusterOutlined', 1, '/ops/device', '',
  '', 'LAYOUT', 1, '', 0, 0,
  '', 0, 0, 0, 1, '', 30, '',
  1, @now, @now
FROM DUAL
WHERE NOT EXISTS (
  SELECT 1 FROM `hg_admin_menu` WHERE `name` = 'opsManage'
);

SET @opsRootId := (SELECT `id` FROM `hg_admin_menu` WHERE `name` = 'opsManage' LIMIT 1);

INSERT INTO `hg_admin_menu` (
  `pid`, `title`, `name`, `path`, `icon`, `type`, `redirect`, `permissions`,
  `permission_name`, `component`, `always_show`, `active_menu`, `is_root`, `is_frame`,
  `frame_src`, `keep_alive`, `hidden`, `affix`, `level`, `tree`, `sort`, `remark`,
  `status`, `created_at`, `updated_at`
)
SELECT
  @opsRootId, '设备管理', 'opsDevice', 'device', '', 2, '', '/opsDevice/list',
  '', '/ops/device/index', 1, 'opsManage', 0, 0,
  '', 0, 0, 0, 2, CONCAT('tr_', @opsRootId, ' '), 10, '',
  1, @now, @now
FROM DUAL
WHERE @opsRootId IS NOT NULL
  AND NOT EXISTS (
    SELECT 1 FROM `hg_admin_menu` WHERE `name` = 'opsDevice'
  );

SET @opsDeviceId := (SELECT `id` FROM `hg_admin_menu` WHERE `name` = 'opsDevice' LIMIT 1);

INSERT INTO `hg_admin_menu` (
  `pid`, `title`, `name`, `path`, `icon`, `type`, `redirect`, `permissions`,
  `permission_name`, `component`, `always_show`, `active_menu`, `is_root`, `is_frame`,
  `frame_src`, `keep_alive`, `hidden`, `affix`, `level`, `tree`, `sort`, `remark`,
  `status`, `created_at`, `updated_at`
)
SELECT
  @opsRootId, '硬件管理', 'opsHardware', 'hardware', '', 2, '', '/opsHardware/overview,/opsHardware/export,/opsDeviceGroup/list,/opsAsset/list',
  '', '/ops/hardware/index', 1, 'opsManage', 0, 0,
  '', 0, 0, 0, 2, CONCAT('tr_', @opsRootId, ' '), 20, '',
  1, @now, @now
FROM DUAL
WHERE @opsRootId IS NOT NULL
  AND NOT EXISTS (
    SELECT 1 FROM `hg_admin_menu` WHERE `name` = 'opsHardware'
  );

SET @opsHardwareId := (SELECT `id` FROM `hg_admin_menu` WHERE `name` = 'opsHardware' LIMIT 1);

INSERT INTO `hg_admin_menu` (
  `pid`, `title`, `name`, `path`, `icon`, `type`, `redirect`, `permissions`,
  `permission_name`, `component`, `always_show`, `active_menu`, `is_root`, `is_frame`,
  `frame_src`, `keep_alive`, `hidden`, `affix`, `level`, `tree`, `sort`, `remark`,
  `status`, `created_at`, `updated_at`
)
SELECT
  @opsDeviceId, '设备详情', 'opsDeviceView', '', '', 3, '', '/opsDevice/view',
  '', '', 1, '', 0, 0,
  '', 0, 1, 0, 3, CONCAT('tr_', @opsRootId, ' tr_', @opsDeviceId, ' '), 10, '',
  1, @now, @now
FROM DUAL
WHERE @opsDeviceId IS NOT NULL
  AND NOT EXISTS (
    SELECT 1 FROM `hg_admin_menu` WHERE `name` = 'opsDeviceView'
  );

INSERT INTO `hg_admin_menu` (
  `pid`, `title`, `name`, `path`, `icon`, `type`, `redirect`, `permissions`,
  `permission_name`, `component`, `always_show`, `active_menu`, `is_root`, `is_frame`,
  `frame_src`, `keep_alive`, `hidden`, `affix`, `level`, `tree`, `sort`, `remark`,
  `status`, `created_at`, `updated_at`
)
SELECT
  @opsDeviceId, '编辑设备', 'opsDeviceEdit', '', '', 3, '', '/opsDevice/edit',
  '', '', 1, '', 0, 0,
  '', 0, 1, 0, 3, CONCAT('tr_', @opsRootId, ' tr_', @opsDeviceId, ' '), 20, '',
  1, @now, @now
FROM DUAL
WHERE @opsDeviceId IS NOT NULL
  AND NOT EXISTS (
    SELECT 1 FROM `hg_admin_menu` WHERE `name` = 'opsDeviceEdit'
  );

SET @opsDeviceEditId := (SELECT `id` FROM `hg_admin_menu` WHERE `name` = 'opsDeviceEdit' LIMIT 1);

INSERT INTO `hg_admin_menu` (
  `pid`, `title`, `name`, `path`, `icon`, `type`, `redirect`, `permissions`,
  `permission_name`, `component`, `always_show`, `active_menu`, `is_root`, `is_frame`,
  `frame_src`, `keep_alive`, `hidden`, `affix`, `level`, `tree`, `sort`, `remark`,
  `status`, `created_at`, `updated_at`
)
SELECT
  @opsDeviceEditId, '设备最大排序', 'opsDeviceMaxSort', '', '', 3, '', '/opsDevice/maxSort',
  '', '', 1, '', 0, 0,
  '', 0, 1, 0, 4, CONCAT('tr_', @opsRootId, ' tr_', @opsDeviceId, ' tr_', @opsDeviceEditId, ' '), 30, '',
  1, @now, @now
FROM DUAL
WHERE @opsDeviceEditId IS NOT NULL
  AND NOT EXISTS (
    SELECT 1 FROM `hg_admin_menu` WHERE `name` = 'opsDeviceMaxSort'
  );

INSERT INTO `hg_admin_menu` (
  `pid`, `title`, `name`, `path`, `icon`, `type`, `redirect`, `permissions`,
  `permission_name`, `component`, `always_show`, `active_menu`, `is_root`, `is_frame`,
  `frame_src`, `keep_alive`, `hidden`, `affix`, `level`, `tree`, `sort`, `remark`,
  `status`, `created_at`, `updated_at`
)
SELECT
  @opsDeviceId, '删除设备', 'opsDeviceDelete', '', '', 3, '', '/opsDevice/delete',
  '', '', 1, '', 0, 0,
  '', 0, 1, 0, 3, CONCAT('tr_', @opsRootId, ' tr_', @opsDeviceId, ' '), 40, '',
  1, @now, @now
FROM DUAL
WHERE @opsDeviceId IS NOT NULL
  AND NOT EXISTS (
    SELECT 1 FROM `hg_admin_menu` WHERE `name` = 'opsDeviceDelete'
  );

INSERT INTO `hg_admin_menu` (
  `pid`, `title`, `name`, `path`, `icon`, `type`, `redirect`, `permissions`,
  `permission_name`, `component`, `always_show`, `active_menu`, `is_root`, `is_frame`,
  `frame_src`, `keep_alive`, `hidden`, `affix`, `level`, `tree`, `sort`, `remark`,
  `status`, `created_at`, `updated_at`
)
SELECT
  @opsDeviceId, '设备状态', 'opsDeviceStatus', '', '', 3, '', '/opsDevice/status',
  '', '', 1, '', 0, 0,
  '', 0, 1, 0, 3, CONCAT('tr_', @opsRootId, ' tr_', @opsDeviceId, ' '), 50, '',
  1, @now, @now
FROM DUAL
WHERE @opsDeviceId IS NOT NULL
  AND NOT EXISTS (
    SELECT 1 FROM `hg_admin_menu` WHERE `name` = 'opsDeviceStatus'
  );

INSERT INTO `hg_admin_menu` (
  `pid`, `title`, `name`, `path`, `icon`, `type`, `redirect`, `permissions`,
  `permission_name`, `component`, `always_show`, `active_menu`, `is_root`, `is_frame`,
  `frame_src`, `keep_alive`, `hidden`, `affix`, `level`, `tree`, `sort`, `remark`,
  `status`, `created_at`, `updated_at`
)
SELECT
  @opsDeviceId, '网络唤醒', 'opsDeviceWake', '', '', 3, '', '/opsDevice/wake',
  '', '', 1, '', 0, 0,
  '', 0, 1, 0, 3, CONCAT('tr_', @opsRootId, ' tr_', @opsDeviceId, ' '), 55, '',
  1, @now, @now
FROM DUAL
WHERE @opsDeviceId IS NOT NULL
  AND NOT EXISTS (
    SELECT 1 FROM `hg_admin_menu` WHERE `name` = 'opsDeviceWake'
  );

INSERT INTO `hg_admin_menu` (
  `pid`, `title`, `name`, `path`, `icon`, `type`, `redirect`, `permissions`,
  `permission_name`, `component`, `always_show`, `active_menu`, `is_root`, `is_frame`,
  `frame_src`, `keep_alive`, `hidden`, `affix`, `level`, `tree`, `sort`, `remark`,
  `status`, `created_at`, `updated_at`
)
SELECT
  @opsDeviceId, '设备选项', 'opsDeviceOption', '', '', 3, '', '/opsDevice/option',
  '', '', 1, '', 0, 0,
  '', 0, 1, 0, 3, CONCAT('tr_', @opsRootId, ' tr_', @opsDeviceId, ' '), 60, '',
  1, @now, @now
FROM DUAL
WHERE @opsDeviceId IS NOT NULL
  AND NOT EXISTS (
    SELECT 1 FROM `hg_admin_menu` WHERE `name` = 'opsDeviceOption'
  );

INSERT INTO `hg_admin_menu` (
  `pid`, `title`, `name`, `path`, `icon`, `type`, `redirect`, `permissions`,
  `permission_name`, `component`, `always_show`, `active_menu`, `is_root`, `is_frame`,
  `frame_src`, `keep_alive`, `hidden`, `affix`, `level`, `tree`, `sort`, `remark`,
  `status`, `created_at`, `updated_at`
)
SELECT
  @opsDeviceId, '设备分组列表', 'opsDeviceGroupList', '', '', 3, '', '/opsDeviceGroup/list',
  '', '', 1, '', 0, 0,
  '', 0, 1, 0, 3, CONCAT('tr_', @opsRootId, ' tr_', @opsDeviceId, ' '), 70, '',
  1, @now, @now
FROM DUAL
WHERE @opsDeviceId IS NOT NULL
  AND NOT EXISTS (
    SELECT 1 FROM `hg_admin_menu` WHERE `name` = 'opsDeviceGroupList'
  );

INSERT INTO `hg_admin_menu` (
  `pid`, `title`, `name`, `path`, `icon`, `type`, `redirect`, `permissions`,
  `permission_name`, `component`, `always_show`, `active_menu`, `is_root`, `is_frame`,
  `frame_src`, `keep_alive`, `hidden`, `affix`, `level`, `tree`, `sort`, `remark`,
  `status`, `created_at`, `updated_at`
)
SELECT
  @opsDeviceId, '设备分组详情', 'opsDeviceGroupView', '', '', 3, '', '/opsDeviceGroup/view',
  '', '', 1, '', 0, 0,
  '', 0, 1, 0, 3, CONCAT('tr_', @opsRootId, ' tr_', @opsDeviceId, ' '), 80, '',
  1, @now, @now
FROM DUAL
WHERE @opsDeviceId IS NOT NULL
  AND NOT EXISTS (
    SELECT 1 FROM `hg_admin_menu` WHERE `name` = 'opsDeviceGroupView'
  );

INSERT INTO `hg_admin_menu` (
  `pid`, `title`, `name`, `path`, `icon`, `type`, `redirect`, `permissions`,
  `permission_name`, `component`, `always_show`, `active_menu`, `is_root`, `is_frame`,
  `frame_src`, `keep_alive`, `hidden`, `affix`, `level`, `tree`, `sort`, `remark`,
  `status`, `created_at`, `updated_at`
)
SELECT
  @opsDeviceId, '编辑设备分组', 'opsDeviceGroupEdit', '', '', 3, '', '/opsDeviceGroup/edit',
  '', '', 1, '', 0, 0,
  '', 0, 1, 0, 3, CONCAT('tr_', @opsRootId, ' tr_', @opsDeviceId, ' '), 90, '',
  1, @now, @now
FROM DUAL
WHERE @opsDeviceId IS NOT NULL
  AND NOT EXISTS (
    SELECT 1 FROM `hg_admin_menu` WHERE `name` = 'opsDeviceGroupEdit'
  );

INSERT INTO `hg_admin_menu` (
  `pid`, `title`, `name`, `path`, `icon`, `type`, `redirect`, `permissions`,
  `permission_name`, `component`, `always_show`, `active_menu`, `is_root`, `is_frame`,
  `frame_src`, `keep_alive`, `hidden`, `affix`, `level`, `tree`, `sort`, `remark`,
  `status`, `created_at`, `updated_at`
)
SELECT
  @opsDeviceId, '设备分组最大排序', 'opsDeviceGroupMaxSort', '', '', 3, '', '/opsDeviceGroup/maxSort',
  '', '', 1, '', 0, 0,
  '', 0, 1, 0, 3, CONCAT('tr_', @opsRootId, ' tr_', @opsDeviceId, ' '), 100, '',
  1, @now, @now
FROM DUAL
WHERE @opsDeviceId IS NOT NULL
  AND NOT EXISTS (
    SELECT 1 FROM `hg_admin_menu` WHERE `name` = 'opsDeviceGroupMaxSort'
  );

INSERT INTO `hg_admin_menu` (
  `pid`, `title`, `name`, `path`, `icon`, `type`, `redirect`, `permissions`,
  `permission_name`, `component`, `always_show`, `active_menu`, `is_root`, `is_frame`,
  `frame_src`, `keep_alive`, `hidden`, `affix`, `level`, `tree`, `sort`, `remark`,
  `status`, `created_at`, `updated_at`
)
SELECT
  @opsDeviceId, '删除设备分组', 'opsDeviceGroupDelete', '', '', 3, '', '/opsDeviceGroup/delete',
  '', '', 1, '', 0, 0,
  '', 0, 1, 0, 3, CONCAT('tr_', @opsRootId, ' tr_', @opsDeviceId, ' '), 110, '',
  1, @now, @now
FROM DUAL
WHERE @opsDeviceId IS NOT NULL
  AND NOT EXISTS (
    SELECT 1 FROM `hg_admin_menu` WHERE `name` = 'opsDeviceGroupDelete'
  );

INSERT INTO `hg_admin_menu` (
  `pid`, `title`, `name`, `path`, `icon`, `type`, `redirect`, `permissions`,
  `permission_name`, `component`, `always_show`, `active_menu`, `is_root`, `is_frame`,
  `frame_src`, `keep_alive`, `hidden`, `affix`, `level`, `tree`, `sort`, `remark`,
  `status`, `created_at`, `updated_at`
)
SELECT
  @opsDeviceId, '设备分组状态', 'opsDeviceGroupStatus', '', '', 3, '', '/opsDeviceGroup/status',
  '', '', 1, '', 0, 0,
  '', 0, 1, 0, 3, CONCAT('tr_', @opsRootId, ' tr_', @opsDeviceId, ' '), 120, '',
  1, @now, @now
FROM DUAL
WHERE @opsDeviceId IS NOT NULL
  AND NOT EXISTS (
    SELECT 1 FROM `hg_admin_menu` WHERE `name` = 'opsDeviceGroupStatus'
  );

INSERT INTO `hg_admin_menu` (
  `pid`, `title`, `name`, `path`, `icon`, `type`, `redirect`, `permissions`,
  `permission_name`, `component`, `always_show`, `active_menu`, `is_root`, `is_frame`,
  `frame_src`, `keep_alive`, `hidden`, `affix`, `level`, `tree`, `sort`, `remark`,
  `status`, `created_at`, `updated_at`
)
SELECT
  @opsDeviceId, '设备分组选项', 'opsDeviceGroupOption', '', '', 3, '', '/opsDeviceGroup/option',
  '', '', 1, '', 0, 0,
  '', 0, 1, 0, 3, CONCAT('tr_', @opsRootId, ' tr_', @opsDeviceId, ' '), 130, '',
  1, @now, @now
FROM DUAL
WHERE @opsDeviceId IS NOT NULL
  AND NOT EXISTS (
    SELECT 1 FROM `hg_admin_menu` WHERE `name` = 'opsDeviceGroupOption'
  );

INSERT INTO `hg_admin_menu` (
  `pid`, `title`, `name`, `path`, `icon`, `type`, `redirect`, `permissions`,
  `permission_name`, `component`, `always_show`, `active_menu`, `is_root`, `is_frame`,
  `frame_src`, `keep_alive`, `hidden`, `affix`, `level`, `tree`, `sort`, `remark`,
  `status`, `created_at`, `updated_at`
)
SELECT
  @opsHardwareId, '硬件详情', 'opsAssetView', '', '', 3, '', '/opsAsset/view',
  '', '', 1, '', 0, 0,
  '', 0, 1, 0, 3, CONCAT('tr_', @opsRootId, ' tr_', @opsHardwareId, ' '), 10, '',
  1, @now, @now
FROM DUAL
WHERE @opsHardwareId IS NOT NULL
  AND NOT EXISTS (
    SELECT 1 FROM `hg_admin_menu` WHERE `name` = 'opsAssetView'
  );

INSERT INTO `hg_admin_menu` (
  `pid`, `title`, `name`, `path`, `icon`, `type`, `redirect`, `permissions`,
  `permission_name`, `component`, `always_show`, `active_menu`, `is_root`, `is_frame`,
  `frame_src`, `keep_alive`, `hidden`, `affix`, `level`, `tree`, `sort`, `remark`,
  `status`, `created_at`, `updated_at`
)
SELECT
  @opsHardwareId, '编辑硬件', 'opsAssetEdit', '', '', 3, '', '/opsAsset/edit',
  '', '', 1, '', 0, 0,
  '', 0, 1, 0, 3, CONCAT('tr_', @opsRootId, ' tr_', @opsHardwareId, ' '), 20, '',
  1, @now, @now
FROM DUAL
WHERE @opsHardwareId IS NOT NULL
  AND NOT EXISTS (
    SELECT 1 FROM `hg_admin_menu` WHERE `name` = 'opsAssetEdit'
  );

SET @opsAssetEditId := (SELECT `id` FROM `hg_admin_menu` WHERE `name` = 'opsAssetEdit' LIMIT 1);

INSERT INTO `hg_admin_menu` (
  `pid`, `title`, `name`, `path`, `icon`, `type`, `redirect`, `permissions`,
  `permission_name`, `component`, `always_show`, `active_menu`, `is_root`, `is_frame`,
  `frame_src`, `keep_alive`, `hidden`, `affix`, `level`, `tree`, `sort`, `remark`,
  `status`, `created_at`, `updated_at`
)
SELECT
  @opsAssetEditId, '硬件最大排序', 'opsAssetMaxSort', '', '', 3, '', '/opsAsset/maxSort',
  '', '', 1, '', 0, 0,
  '', 0, 1, 0, 4, CONCAT('tr_', @opsRootId, ' tr_', @opsHardwareId, ' tr_', @opsAssetEditId, ' '), 30, '',
  1, @now, @now
FROM DUAL
WHERE @opsAssetEditId IS NOT NULL
  AND NOT EXISTS (
    SELECT 1 FROM `hg_admin_menu` WHERE `name` = 'opsAssetMaxSort'
  );

INSERT INTO `hg_admin_menu` (
  `pid`, `title`, `name`, `path`, `icon`, `type`, `redirect`, `permissions`,
  `permission_name`, `component`, `always_show`, `active_menu`, `is_root`, `is_frame`,
  `frame_src`, `keep_alive`, `hidden`, `affix`, `level`, `tree`, `sort`, `remark`,
  `status`, `created_at`, `updated_at`
)
SELECT
  @opsHardwareId, '删除硬件', 'opsAssetDelete', '', '', 3, '', '/opsAsset/delete',
  '', '', 1, '', 0, 0,
  '', 0, 1, 0, 3, CONCAT('tr_', @opsRootId, ' tr_', @opsHardwareId, ' '), 40, '',
  1, @now, @now
FROM DUAL
WHERE @opsHardwareId IS NOT NULL
  AND NOT EXISTS (
    SELECT 1 FROM `hg_admin_menu` WHERE `name` = 'opsAssetDelete'
  );

INSERT INTO `hg_admin_menu` (
  `pid`, `title`, `name`, `path`, `icon`, `type`, `redirect`, `permissions`,
  `permission_name`, `component`, `always_show`, `active_menu`, `is_root`, `is_frame`,
  `frame_src`, `keep_alive`, `hidden`, `affix`, `level`, `tree`, `sort`, `remark`,
  `status`, `created_at`, `updated_at`
)
SELECT
  @opsHardwareId, '硬件状态', 'opsAssetStatus', '', '', 3, '', '/opsAsset/status',
  '', '', 1, '', 0, 0,
  '', 0, 1, 0, 3, CONCAT('tr_', @opsRootId, ' tr_', @opsHardwareId, ' '), 50, '',
  1, @now, @now
FROM DUAL
WHERE @opsHardwareId IS NOT NULL
  AND NOT EXISTS (
    SELECT 1 FROM `hg_admin_menu` WHERE `name` = 'opsAssetStatus'
  );

INSERT INTO `hg_admin_role_menu` (`role_id`, `menu_id`)
SELECT r.`id`, m.`id`
FROM `hg_admin_role` r
JOIN `hg_admin_menu` m
WHERE r.`key` IN ('super', 'manage')
  AND m.`name` IN (
    'opsManage',
    'opsDevice',
    'opsHardware',
    'opsDeviceView',
    'opsDeviceEdit',
    'opsDeviceMaxSort',
    'opsDeviceDelete',
    'opsDeviceStatus',
    'opsDeviceWake',
    'opsDeviceOption',
    'opsDeviceGroupList',
    'opsDeviceGroupView',
    'opsDeviceGroupEdit',
    'opsDeviceGroupMaxSort',
    'opsDeviceGroupDelete',
    'opsDeviceGroupStatus',
    'opsDeviceGroupOption',
    'opsAssetView',
    'opsAssetEdit',
    'opsAssetMaxSort',
    'opsAssetDelete',
    'opsAssetStatus'
  )
  AND NOT EXISTS (
    SELECT 1
    FROM `hg_admin_role_menu` rm
    WHERE rm.`role_id` = r.`id`
      AND rm.`menu_id` = m.`id`
  );

UPDATE `hg_ops_asset`
SET `asset_name` = '网卡'
WHERE `asset_type` = 'network'
  AND `asset_name` = 'Network Interface';

UPDATE `hg_admin_menu`
SET `title` = '硬件管理'
WHERE `name` = 'opsHardware';

UPDATE `hg_admin_menu`
SET `permissions` = '/opsHardware/overview,/opsHardware/export,/opsDeviceGroup/list,/opsAsset/list'
WHERE `name` = 'opsHardware';

UPDATE `hg_admin_menu`
SET `title` = '硬件详情'
WHERE `name` = 'opsAssetView';

UPDATE `hg_admin_menu`
SET `title` = '编辑硬件'
WHERE `name` = 'opsAssetEdit';

UPDATE `hg_admin_menu`
SET `title` = '硬件最大排序'
WHERE `name` = 'opsAssetMaxSort';

UPDATE `hg_admin_menu`
SET `title` = '删除硬件'
WHERE `name` = 'opsAssetDelete';

UPDATE `hg_admin_menu`
SET `title` = '硬件状态'
WHERE `name` = 'opsAssetStatus';

COMMIT;

SET FOREIGN_KEY_CHECKS = 1;
