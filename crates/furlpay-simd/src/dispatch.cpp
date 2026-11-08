// FurlPay SIMD — Runtime dispatch to the best available kernel.

#include "furlpay_simd.h"

// Forward declaration of scalar fallback
extern "C" size_t furlpay_intersect_scalar(
    const uint32_t* small_arr, size_t small_sz,
    const uint32_t* large_arr, size_t large_sz,
    uint32_t* out_buf, size_t out_cap
);

extern "C" size_t furlpay_intersect_simd(
    const uint32_t* small_arr, size_t small_sz,
    const uint32_t* large_arr, size_t large_sz,
    uint32_t* out_buf, size_t out_cap
) {
    if (!small_arr || !large_arr || !out_buf || out_cap == 0 || small_sz == 0 || large_sz == 0) {
        return 0;
    }
    // Phase 3 will add runtime dispatch based on furlpay_detect_cpu().
    // For now, all platforms use the galloping scalar baseline with hardware capacity bounds.
    return furlpay_intersect_scalar(small_arr, small_sz, large_arr, large_sz, out_buf, out_cap);
}
