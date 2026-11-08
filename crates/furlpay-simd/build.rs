//! Build script for compiling C++ SIMD kernels via the cc crate.

fn main() {
    let mut build = cc::Build::new();
    build
        .cpp(true)
        .include("include")
        .warnings(true)
        .extra_warnings(true);

    // Set C++ standard
    if cfg!(target_env = "msvc") {
        build.flag("/std:c++20");
        build.flag("/EHsc");
    } else {
        build.flag("-std=c++20");
    }

    // Core files: scalar baseline, CPU detection, and dispatch
    build.file("src/intersect_scalar.cpp");
    build.file("src/cpu_detect.cpp");
    build.file("src/dispatch.cpp");

    build.compile("furlpay_simd_core");

    // Conditionally compile platform-specific kernels
    if cfg!(target_arch = "x86_64") {
        // AVX2 kernel
        let mut avx2_build = cc::Build::new();
        avx2_build.cpp(true).include("include").file("src/intersect_avx2.cpp");
        if cfg!(target_env = "msvc") {
            avx2_build.flag("/std:c++20").flag("/arch:AVX2").flag("/EHsc");
        } else {
            avx2_build.flag("-std=c++20").flag("-mavx2");
        }
        avx2_build.compile("furlpay_avx2");

        // AVX-512 kernel
        let mut avx512_build = cc::Build::new();
        avx512_build.cpp(true).include("include").file("src/intersect_avx512.cpp");
        if cfg!(target_env = "msvc") {
            avx512_build.flag("/std:c++20").flag("/arch:AVX512").flag("/EHsc");
        } else {
            avx512_build
                .flag("-std=c++20")
                .flag("-mavx512f")
                .flag("-mavx512bw")
                .flag("-mavx512dq");
        }
        avx512_build.compile("furlpay_avx512");
    }

    println!("cargo:rerun-if-changed=src/cpu_detect.cpp");
    println!("cargo:rerun-if-changed=src/intersect_scalar.cpp");
    println!("cargo:rerun-if-changed=src/intersect_avx2.cpp");
    println!("cargo:rerun-if-changed=src/intersect_avx512.cpp");
    println!("cargo:rerun-if-changed=src/intersect_neon.cpp");
    println!("cargo:rerun-if-changed=src/dispatch.cpp");
    println!("cargo:rerun-if-changed=include/furlpay_simd.h");
}
