// FurlPay SIMD — Two-pointer galloping intersection baseline.

#include "furlpay_simd.h"
#include <algorithm>
#include <cstddef>

static size_t gallop_search(const uint32_t* arr, size_t len, size_t start, uint32_t target) {
    size_t lo = start;
    size_t bound = 1;

    while (lo + bound < len && arr[lo + bound] < target) {
        bound *= 2;
    }

    size_t hi = std::min(lo + bound, len);
    lo = lo + (bound / 2);
    while (lo < hi) {
        size_t mid = lo + (hi - lo) / 2;
        if (arr[mid] < target) {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    return lo;
}

extern "C" size_t furlpay_intersect_scalar(
    const uint32_t* small_arr, size_t small_sz,
    const uint32_t* large_arr, size_t large_sz,
    uint32_t* out_buf, size_t out_cap
) {
    if (!small_arr || !large_arr || !out_buf || out_cap == 0 || small_sz == 0 || large_sz == 0) {
        return 0;
    }

    size_t out_count = 0;
    size_t j = 0;

    for (size_t i = 0; i < small_sz && out_count < out_cap; ++i) {
        uint32_t target = small_arr[i];

        if (large_sz > 8 * small_sz) {
            j = gallop_search(large_arr, large_sz, j, target);
        } else {
            while (j < large_sz && large_arr[j] < target) {
                ++j;
            }
        }

        if (j < large_sz && large_arr[j] == target) {
            out_buf[out_count++] = target;
            ++j;
        }
    }

    return out_count;
}
