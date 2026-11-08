//! FurlPay SIMD — Safe Rust wrapper for multi-target SIMD intersection kernels.

/// CPU architecture enum matching the C side.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum CpuArch {
    Scalar = 0,
    X86Avx2 = 1,
    X86Avx512 = 2,
    ArmNeon = 3,
}

impl CpuArch {
    /// Human-readable name for the architecture.
    pub fn name(&self) -> &'static str {
        match self {
            CpuArch::Scalar => "Scalar (baseline)",
            CpuArch::X86Avx2 => "x86-64 AVX2 (256-bit)",
            CpuArch::X86Avx512 => "x86-64 AVX-512 (512-bit)",
            CpuArch::ArmNeon => "ARM NEON (128-bit)",
        }
    }
}

impl From<i32> for CpuArch {
    fn from(v: i32) -> Self {
        match v {
            1 => CpuArch::X86Avx2,
            2 => CpuArch::X86Avx512,
            3 => CpuArch::ArmNeon,
            _ => CpuArch::Scalar,
        }
    }
}

extern "C" {
    fn furlpay_detect_cpu() -> i32;
    fn furlpay_intersect_simd(
        small_arr: *const u32,
        small_sz: usize,
        large_arr: *const u32,
        large_sz: usize,
        out_buf: *mut u32,
        out_cap: usize,
    ) -> usize;
}

/// Detect the CPU architecture at runtime.
pub fn detect_cpu() -> CpuArch {
    CpuArch::from(unsafe { furlpay_detect_cpu() })
}

/// Intersect two sorted posting lists using the best available SIMD kernel.
///
/// # Panics
/// Panics if the C++ kernel writes more elements than the output buffer capacity.
pub fn intersect_posting_lists(a: &[u32], b: &[u32], output: &mut Vec<u32>) {
    if a.is_empty() || b.is_empty() {
        output.clear();
        return;
    }

    let (small, large) = if a.len() <= b.len() { (a, b) } else { (b, a) };

    output.clear();
    output.reserve(small.len());

    let capacity = output.capacity();
    let written = unsafe {
        furlpay_intersect_simd(
            small.as_ptr(),
            small.len(),
            large.as_ptr(),
            large.len(),
            output.as_mut_ptr(),
            capacity,
        )
    };

    // HARD SECURITY ASSERTION: Prevent C++ kernel buffer overflow.
    assert!(
        written <= capacity,
        "FATAL: SIMD intersection kernel wrote {} elements into buffer of capacity {}",
        written,
        capacity
    );

    unsafe {
        output.set_len(written);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_cpu() {
        let arch = detect_cpu();
        println!("Detected CPU: {:?} ({})", arch, arch.name());
        assert!(matches!(
            arch,
            CpuArch::Scalar | CpuArch::X86Avx2 | CpuArch::X86Avx512 | CpuArch::ArmNeon
        ));
    }

    #[test]
    fn test_intersect_empty() {
        let mut output = Vec::new();
        intersect_posting_lists(&[], &[1, 2, 3], &mut output);
        assert!(output.is_empty());
    }

    #[test]
    fn test_intersect_disjoint() {
        let mut output = Vec::new();
        intersect_posting_lists(&[1, 3, 5], &[2, 4, 6], &mut output);
        assert!(output.is_empty());
    }

    #[test]
    fn test_intersect_overlap() {
        let mut output = Vec::new();
        intersect_posting_lists(&[1, 2, 3, 5, 8], &[2, 3, 7, 8, 9], &mut output);
        assert_eq!(output, vec![2, 3, 8]);
    }
}
