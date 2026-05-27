#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unsafe_op_in_unsafe_fn)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(windows)]
pub const FFLocalIpType_FF_LOCALIP_TYPE_IPV4_BIT: FFLocalIpType =
    FF_LOCALIP_TYPE_IPV4_BIT as FFLocalIpType;
#[cfg(windows)]
pub const FFLocalIpType_FF_LOCALIP_TYPE_IPV6_BIT: FFLocalIpType =
    FF_LOCALIP_TYPE_IPV6_BIT as FFLocalIpType;
#[cfg(windows)]
pub const FFLocalIpType_FF_LOCALIP_TYPE_MAC_BIT: FFLocalIpType =
    FF_LOCALIP_TYPE_MAC_BIT as FFLocalIpType;
#[cfg(windows)]
pub const FFLocalIpType_FF_LOCALIP_TYPE_MTU_BIT: FFLocalIpType =
    FF_LOCALIP_TYPE_MTU_BIT as FFLocalIpType;
#[cfg(windows)]
pub const FFLocalIpType_FF_LOCALIP_TYPE_SPEED_BIT: FFLocalIpType =
    FF_LOCALIP_TYPE_SPEED_BIT as FFLocalIpType;
#[cfg(windows)]
pub const FFLocalIpType_FF_LOCALIP_TYPE_FLAGS_BIT: FFLocalIpType =
    FF_LOCALIP_TYPE_FLAGS_BIT as FFLocalIpType;
