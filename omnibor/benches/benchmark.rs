//! Benchmarks comparing cryptography backends.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use omnibor::ArtifactIdBuilder;

#[cfg(not(any(
    feature = "provider-rustcrypto",
    feature = "provider-boringssl",
    feature = "provider-openssl"
)))]
compile_error!(
    r#"At least one cryptography backend must be active: "#
    r#""provider-rustcrypto", "provider-boringssl", "provider-openssl""#
);

/*===============================================================================================
 * BENCHMARK FUNCTIONS
 *
 * Define the benchmark functions based on the selected features.
 */

#[cfg(feature = "provider-rustcrypto")]
fn bench_rustcrypto_sha256_small(c: &mut Criterion) {
    let name = "OmniBOR RustCrypto SHA-256 11B";
    let input = b"hello world";
    c.bench_function(name, |b| {
        b.iter(|| {
            let _ = ArtifactIdBuilder::with_rustcrypto().identify_bytes(black_box(input));
        })
    });
}

#[cfg(feature = "provider-boringssl")]
fn bench_boring_sha256_small(c: &mut Criterion) {
    let name = "OmniBOR BoringSSL SHA-256 11B";
    let input = b"hello world";
    c.bench_function(name, |b| {
        b.iter(|| {
            let _ = ArtifactIdBuilder::with_boringssl().identify_bytes(black_box(input));
        })
    });
}

#[cfg(feature = "provider-openssl")]
fn bench_openssl_sha256_small(c: &mut Criterion) {
    let name = "OmniBOR OpenSSL SHA-256 11B";
    let input = b"hello world";
    c.bench_function(name, |b| {
        b.iter(|| {
            let _ = ArtifactIdBuilder::with_openssl().identify_bytes(black_box(input));
        })
    });
}

#[cfg(feature = "provider-rustcrypto")]
fn bench_rustcrypto_sha256_large(c: &mut Criterion) {
    let name = "OmniBOR RustCrypto SHA-256 100MB";
    let input = &[0; 1024 * 1024 * 100]; // 100 MB
    c.bench_function(name, |b| {
        b.iter(|| {
            let _ = ArtifactIdBuilder::with_rustcrypto().identify_bytes(black_box(input));
        })
    });
}

#[cfg(feature = "provider-boringssl")]
fn bench_boring_sha256_large(c: &mut Criterion) {
    let name = "OmniBOR BoringSSL SHA-256 100MB";
    let input = &[0; 1024 * 1024 * 100]; // 100 MB
    c.bench_function(name, |b| {
        b.iter(|| {
            let _ = ArtifactIdBuilder::with_boringssl().identify_bytes(black_box(input));
        })
    });
}

#[cfg(feature = "provider-openssl")]
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

#[cfg(feature = "provider-rustcrypto")]
criterion_group!(
    name = rustcrypto_benches;
    config = Criterion::default();
    targets = bench_rustcrypto_sha256_small, bench_rustcrypto_sha256_large
);

#[cfg(feature = "provider-boringssl")]
criterion_group!(
    name = boringssl_benches;
    config = Criterion::default();
    targets = bench_boring_sha256_small, bench_boring_sha256_large
);

#[cfg(feature = "provider-openssl")]
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
    feature = "provider-rustcrypto",
    feature = "provider-boringssl",
    feature = "provider-openssl"
))]
criterion_main!(rustcrypto_benches, boringssl_benches, openssl_benches);

#[cfg(all(
    feature = "provider-rustcrypto",
    feature = "provider-boringssl",
    not(feature = "provider-openssl")
))]
criterion_main!(rustcrypto_benches, boringssl_benches);

#[cfg(all(
    not(feature = "provider-rustcrypto"),
    feature = "provider-boringssl",
    feature = "provider-openssl"
))]
criterion_main!(boringssl_benches, openssl_benches);

#[cfg(all(
    feature = "provider-rustcrypto",
    not(feature = "provider-boringssl"),
    feature = "provider-openssl"
))]
criterion_main!(rustcrypto_benches, openssl_benches);

#[cfg(all(
    feature = "provider-rustcrypto",
    not(feature = "provider-boringssl"),
    not(feature = "provider-openssl"),
))]
criterion_main!(rustcrypto_benches);

#[cfg(all(
    not(feature = "provider-rustcrypto"),
    feature = "provider-boringssl",
    not(feature = "provider-openssl")
))]
criterion_main!(boringssl_benches);

#[cfg(all(
    not(feature = "provider-rustcrypto"),
    not(feature = "provider-boringssl"),
    feature = "provider-openssl"
))]
criterion_main!(openssl_benches);
