// FurlPay SIMD — CPUID + OSXSAVE + XCR0 feature detection.

#include "furlpay_simd.h"

#if defined(_MSC_VER)
    #include <intrin.h>
#elif defined(__x86_64__) || defined(__i386__)
    #include <cpuid.h>
    #include <immintrin.h>
#endif

extern "C" FurlpayCpuArch furlpay_detect_cpu(void) {
#if defined(__x86_64__) || defined(_M_X64)
    uint32_t eax, ebx, ecx, edx;

    #if defined(_MSC_VER)
        int info[4];
        __cpuid(info, 1);
        ecx = static_cast<uint32_t>(info[2]);
    #else
        __cpuid(1, eax, ebx, ecx, edx);
    #endif

    bool has_osxsave = (ecx & (1u << 27)) != 0;
    if (!has_osxsave) {
        return CPU_ARCH_SCALAR;
    }

    uint64_t xcr0 = _xgetbv(0);
    bool avx_ready = (xcr0 & 0x6) == 0x6;
    if (!avx_ready) {
        return CPU_ARCH_SCALAR;
    }

    #if defined(_MSC_VER)
        __cpuidex(info, 7, 0);
        ebx = static_cast<uint32_t>(info[1]);
    #else
        __cpuid_count(7, 0, eax, ebx, ecx, edx);
    #endif

    bool has_avx2     = (ebx & (1u << 5))  != 0;
    bool has_avx512f  = (ebx & (1u << 16)) != 0;
    bool has_avx512cd = (ebx & (1u << 28)) != 0;
    bool has_avx512dq = (ebx & (1u << 17)) != 0;
    bool has_avx512bw = (ebx & (1u << 30)) != 0;

    bool avx512_ready = (xcr0 & 0xE6) == 0xE6;

    if (avx512_ready && has_avx512f && has_avx512cd && has_avx512dq && has_avx512bw) {
        return CPU_ARCH_X86_AVX512;
    }

    if (has_avx2) {
        return CPU_ARCH_X86_AVX2;
    }

#elif defined(__aarch64__) || defined(_M_ARM64)
    return CPU_ARCH_ARM_NEON;
#endif

    return CPU_ARCH_SCALAR;
}
