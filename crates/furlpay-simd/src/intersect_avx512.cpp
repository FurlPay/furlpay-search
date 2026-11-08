// FurlPay SIMD — AVX-512 16-lane intersection kernel stub.
// Compiled with -mavx512f -mavx512bw -mavx512dq (GCC/Clang) or /arch:AVX512 (MSVC).

#include "furlpay_simd.h"

#if defined(__x86_64__) || defined(_M_X64)
#include <immintrin.h>

// AVX-512 vectorized intersection — Phase 3 will add vpcompressd-based merge.
// For now, this file compiles with AVX-512 flags to validate the build pipeline.

#endif // x86_64
