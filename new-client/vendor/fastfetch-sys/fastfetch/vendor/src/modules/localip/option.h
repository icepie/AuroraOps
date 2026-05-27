#pragma once

// This file will be included in "fastfetch.h", do NOT put unnecessary things here

#include "common/option.h"

#if defined(_MSC_VER)
typedef uint16_t FFLocalIpType;
#define FF_LOCALIP_TYPE_NONE ((FFLocalIpType) 0)
#define FF_LOCALIP_TYPE_LOOP_BIT ((FFLocalIpType) (1 << 0))
#define FF_LOCALIP_TYPE_IPV4_BIT ((FFLocalIpType) (1 << 1))
#define FF_LOCALIP_TYPE_IPV6_BIT ((FFLocalIpType) (1 << 2))
#define FF_LOCALIP_TYPE_MAC_BIT ((FFLocalIpType) (1 << 3))
#define FF_LOCALIP_TYPE_PREFIX_LEN_BIT ((FFLocalIpType) (1 << 4))
#define FF_LOCALIP_TYPE_MTU_BIT ((FFLocalIpType) (1 << 5))
#define FF_LOCALIP_TYPE_SPEED_BIT ((FFLocalIpType) (1 << 6))
#define FF_LOCALIP_TYPE_FLAGS_BIT ((FFLocalIpType) (1 << 7))
#define FF_LOCALIP_TYPE_COMPACT_BIT ((FFLocalIpType) (1 << 10))
#define FF_LOCALIP_TYPE_DEFAULT_ROUTE_ONLY_BIT ((FFLocalIpType) (1 << 11))
#define FF_LOCALIP_TYPE_ALL_IPS_BIT ((FFLocalIpType) (1 << 12))
#define FF_LOCALIP_TYPE_FORCE_UNSIGNED UINT16_MAX
#else
typedef enum __attribute__((__packed__)) FFLocalIpType
{
    FF_LOCALIP_TYPE_NONE,
    FF_LOCALIP_TYPE_LOOP_BIT        = 1 << 0,
    FF_LOCALIP_TYPE_IPV4_BIT        = 1 << 1,
    FF_LOCALIP_TYPE_IPV6_BIT        = 1 << 2,
    FF_LOCALIP_TYPE_MAC_BIT         = 1 << 3,
    FF_LOCALIP_TYPE_PREFIX_LEN_BIT  = 1 << 4,
    FF_LOCALIP_TYPE_MTU_BIT  = 1 << 5,
    FF_LOCALIP_TYPE_SPEED_BIT  = 1 << 6,
    FF_LOCALIP_TYPE_FLAGS_BIT  = 1 << 7,

    FF_LOCALIP_TYPE_COMPACT_BIT            = 1 << 10,
    FF_LOCALIP_TYPE_DEFAULT_ROUTE_ONLY_BIT = 1 << 11,
    FF_LOCALIP_TYPE_ALL_IPS_BIT            = 1 << 12,
    FF_LOCALIP_TYPE_FORCE_UNSIGNED         = UINT16_MAX,
} FFLocalIpType;
#endif
static_assert(sizeof(FFLocalIpType) == sizeof(uint16_t), "");

typedef struct FFLocalIpOptions
{
    FFModuleBaseInfo moduleInfo;
    FFModuleArgs moduleArgs;

    FFLocalIpType showType;
    FFstrbuf namePrefix;
} FFLocalIpOptions;
