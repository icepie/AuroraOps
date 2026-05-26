#include "gpu_driver_specific.h"

const char* ffDetectIntelGpuInfo(
    const FFGpuDriverCondition* cond,
    FFGpuDriverResult result,
    const char* soName
) {
    (void) cond;
    (void) result;
    (void) soName;
    return "Intel GPU driver extension disabled";
}

const char* ffDetectAmdGpuInfo(
    const FFGpuDriverCondition* cond,
    FFGpuDriverResult result,
    const char* soName
) {
    (void) cond;
    (void) result;
    (void) soName;
    return "AMD GPU driver extension disabled";
}
