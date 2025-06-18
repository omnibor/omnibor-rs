//! Benchmarks comparing cryptography backends.

use criterion::{criterion_group, criterion_main, Criterion};
use omnibor::ArtifactIdBuilder;
use std::hint::black_box;

#[cfg(not(any(
    feature = "backend-rustcrypto",
    feature = "backend-boringssl",
    feature = "backend-openssl"
)))]
compile_error!(
    r#"At least one cryptography backend must be active: "#
    r#""backend-rustcrypto", "backend-boringssl", "backend-openssl""#
);

/*===============================================================================================
 * BENCHMARK FUNCTIONS
 *
 * Define the benchmark functions based on the selected features.
 */

#[cfg(feature = "backend-rustcrypto")]
fn bench_rustcrypto_sha256_small(c: &mut Criterion) {
    let name = "OmniBOR RustCrypto SHA-256 11B";
    let input = b"hello world";
    c.bench_function(name, |b| {
        b.iter(|| {
            let _ = ArtifactIdBuilder::with_rustcrypto().identify_bytes(black_box(input));
        })
    });
}

#[cfg(feature = "backend-boringssl")]
fn bench_boring_sha256_small(c: &mut Criterion) {
    let name = "OmniBOR BoringSSL SHA-256 11B";
    let input = b"hello world";
    c.bench_function(name, |b| {
        b.iter(|| {
            let _ = ArtifactIdBuilder::with_boringssl().identify_bytes(black_box(input));
        })
    });
}

#[cfg(feature = "backend-openssl")]
fn bench_openssl_sha256_small(c: &mut Criterion) {
    let name = "OmniBOR OpenSSL SHA-256 11B";
    let input = b"hello world";
    c.bench_function(name, |b| {
        b.iter(|| {
            let _ = ArtifactIdBuilder::with_openssl().identify_bytes(black_box(input));
        })
    });
}

#[cfg(feature = "backend-rustcrypto")]
fn bench_rustcrypto_sha256_large(c: &mut Criterion) {
    let name = "OmniBOR RustCrypto SHA-256 100MB";
    let input = &[0; 1024 * 1024 * 100]; // 100 MB
    c.bench_function(name, |b| {
        b.iter(|| {
            let _ = ArtifactIdBuilder::with_rustcrypto().identify_bytes(black_box(input));
        })
    });
}

#[cfg(feature = "backend-boringssl")]
fn bench_boring_sha256_large(c: &mut Criterion) {
    let name = "OmniBOR BoringSSL SHA-256 100MB";
    let input = &[0; 1024 * 1024 * 100]; // 100 MB
    c.bench_function(name, |b| {
        b.iter(|| {
            let _ = ArtifactIdBuilder::with_boringssl().identify_bytes(black_box(input));
        })
    });
}

#[cfg(feature = "backend-openssl")]
fn bench_openssl_sha256_large(c: &mut Criterion) {
    let name = "OmniBOR OpenSSL SHA-256 100MB";
    let input = &[0; 1024 * 1024 * 100]; // 100 MB
    c.bench_function(name, |b| {
        b.iter(|| {
            let _ = ArtifactIdBuilder::with_openssl().identify_bytes(black_box(input));
        })
    });
}

/*===============================================================================================
 * BENCHMARK GROUPS
 *
 * Define the benchmark groups based on the selected features.
 */

#[cfg(feature = "backend-rustcrypto")]
criterion_group!(
    name = rustcrypto_benches;
    config = Criterion::default();
    targets = bench_rustcrypto_sha256_small, bench_rustcrypto_sha256_large
);

#[cfg(feature = "backend-boringssl")]
criterion_group!(
    name = boringssl_benches;
    config = Criterion::default();
    targets = bench_boring_sha256_small, bench_boring_sha256_large
);

#[cfg(feature = "backend-openssl")]
criterion_group!(
    name = openssl_benches;
    config = Criterion::default();
    targets = bench_openssl_sha256_small, bench_openssl_sha256_large
);

/*===============================================================================================
 * MAIN FUNCTION
 *
 * Use conditional compilation to select the main function, incorporating the defined benchmark
 * groups based on the selected features.
 */

#[cfg(all(
    feature = "backend-rustcrypto",
    feature = "backend-boringssl",
    feature = "backend-openssl"
))]
criterion_main!(rustcrypto_benches, boringssl_benches, openssl_benches);

#[cfg(all(
    feature = "backend-rustcrypto",
    feature = "backend-boringssl",
    not(feature = "backend-openssl")
))]
criterion_main!(rustcrypto_benches, boringssl_benches);

#[cfg(all(
    not(feature = "backend-rustcrypto"),
    feature = "backend-boringssl",
    feature = "backend-openssl"
))]
criterion_main!(boringssl_benches, openssl_benches);

#[cfg(all(
    feature = "backend-rustcrypto",
    not(feature = "backend-boringssl"),
    feature = "backend-openssl"
))]
criterion_main!(rustcrypto_benches, openssl_benches);

#[cfg(all(
    feature = "backend-rustcrypto",
    not(feature = "backend-boringssl"),
    not(feature = "backend-openssl"),
))]
criterion_main!(rustcrypto_benches);

#[cfg(all(
    not(feature = "backend-rustcrypto"),
    feature = "backend-boringssl",
    not(feature = "backend-openssl")
))]
criterion_main!(boringssl_benches);

#[cfg(all(
    not(feature = "backend-rustcrypto"),
    not(feature = "backend-boringssl"),
    feature = "backend-openssl"
))]
criterion_main!(openssl_benches);
