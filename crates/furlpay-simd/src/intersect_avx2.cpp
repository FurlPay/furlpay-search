// FurlPay SIMD — AVX2 8-lane intersection kernel stub.
// Compiled with -mavx2 (GCC/Clang) or /arch:AVX2 (MSVC).

#include "furlpay_simd.h"

#if defined(__x86_64__) || defined(_M_X64)
#include <immintrin.h>

// AVX2 vectorized intersection — Phase 3 will add full SIMD merge.
// For now, this file compiles with AVX2 flags to validate the build pipeline.

#endif // x86_64
