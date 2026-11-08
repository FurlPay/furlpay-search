// FurlPay SIMD — C ABI header for CPU detection and posting list intersection.
#ifndef FURLPAY_SIMD_H
#define FURLPAY_SIMD_H

#include <cstddef>
#include <cstdint>

#ifdef __cplusplus
extern "C" {
#endif

/// CPU architecture detected at runtime.
typedef enum {
    CPU_ARCH_SCALAR     = 0,
    CPU_ARCH_X86_AVX2   = 1,
    CPU_ARCH_X86_AVX512 = 2,
    CPU_ARCH_ARM_NEON   = 3,
} FurlpayCpuArch;

/// Detect the best available CPU SIMD architecture.
FurlpayCpuArch furlpay_detect_cpu(void);

/// Intersect two sorted u32 arrays using the best available SIMD kernel.
size_t furlpay_intersect_simd(
    const uint32_t* small_arr, size_t small_sz,
    const uint32_t* large_arr, size_t large_sz,
    uint32_t* out_buf, size_t out_cap
);

/// Scalar two-pointer galloping intersection (reference baseline).
size_t furlpay_intersect_scalar(
    const uint32_t* small_arr, size_t small_sz,
    const uint32_t* large_arr, size_t large_sz,
    uint32_t* out_buf, size_t out_cap
);

#ifdef __cplusplus
}
#endif

#endif // FURLPAY_SIMD_H
