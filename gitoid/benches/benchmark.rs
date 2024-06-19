use criterion::black_box;
use criterion::criterion_group;
use criterion::criterion_main;
use criterion::Criterion;
#[cfg(feature = "boringssl")]
use gitoid::boringssl::Sha1 as BoringSha1;
#[cfg(feature = "boringssl")]
use gitoid::boringssl::Sha256 as BoringSha256;
#[cfg(all(feature = "rustcrypto", feature = "sha1"))]
use gitoid::rustcrypto::Sha1 as RustSha1;
#[cfg(all(feature = "rustcrypto", feature = "sha256"))]
use gitoid::rustcrypto::Sha256 as RustSha256;
use gitoid::Blob;
use gitoid::GitOid;

#[cfg(not(any(feature = "rustcrypto", feature = "boringssl",)))]
compile_error!(
    r#"At least one cryptography backend must be active: "rustcrypto" and/or "boringssl""#
);

#[cfg(feature = "rustcrypto")]
fn bench_rustcrypto_sha1_small(c: &mut Criterion) {
    let name = "GitOid RustCrypto SHA-1 11B";
    let input = b"hello world";
    c.bench_function(name, |b| {
        b.iter(|| {
            let _ = GitOid::<RustSha1, Blob>::id_bytes(black_box(input));
        })
    });
}

#[cfg(feature = "boringssl")]
fn bench_boring_sha1_small(c: &mut Criterion) {
    let name = "GitOid BoringSSL SHA-1 11B";
    let input = b"hello world";
    c.bench_function(name, |b| {
        b.iter(|| {
            let _ = GitOid::<BoringSha1, Blob>::id_bytes(black_box(input));
        })
    });
}

#[cfg(feature = "rustcrypto")]
fn bench_rustcrypto_sha256_small(c: &mut Criterion) {
    let name = "GitOid RustCrypto SHA-256 11B";
    let input = b"hello world";
    c.bench_function(name, |b| {
        b.iter(|| {
            let _ = GitOid::<RustSha256, Blob>::id_bytes(black_box(input));
        })
    });
}

#[cfg(feature = "boringssl")]
fn bench_boring_sha256_small(c: &mut Criterion) {
    let name = "GitOid BoringSSL SHA-256 11B";
    let input = b"hello world";
    c.bench_function(name, |b| {
        b.iter(|| {
            let _ = GitOid::<BoringSha256, Blob>::id_bytes(black_box(input));
        })
    });
}

#[cfg(feature = "rustcrypto")]
fn bench_rustcrypto_sha1_large(c: &mut Criterion) {
    let name = "GitOid RustCrypto SHA-1 100MB";
    let input = &[0; 1024 * 1024 * 100]; // 100 MB
    c.bench_function(name, |b| {
        b.iter(|| {
            let _ = GitOid::<RustSha1, Blob>::id_bytes(black_box(input));
        })
    });
}

#[cfg(feature = "boringssl")]
fn bench_boring_sha1_large(c: &mut Criterion) {
    let name = "GitOid BoringSSL SHA-1 100MB";
    let input = &[0; 1024 * 1024 * 100]; // 100 MB
    c.bench_function(name, |b| {
        b.iter(|| {
            let _ = GitOid::<BoringSha1, Blob>::id_bytes(black_box(input));
        })
    });
}

#[cfg(feature = "rustcrypto")]
fn bench_rustcrypto_sha256_large(c: &mut Criterion) {
    let name = "GitOid RustCrypto SHA-256 100MB";
    let input = &[0; 1024 * 1024 * 100]; // 100 MB
    c.bench_function(name, |b| {
        b.iter(|| {
            let _ = GitOid::<RustSha256, Blob>::id_bytes(black_box(input));
        })
    });
}

#[cfg(feature = "boringssl")]
fn bench_boring_sha256_large(c: &mut Criterion) {
    let name = "GitOid BoringSSL SHA-256 100MB";
    let input = &[0; 1024 * 1024 * 100]; // 100 MB
    c.bench_function(name, |b| {
        b.iter(|| {
            let _ = GitOid::<BoringSha256, Blob>::id_bytes(black_box(input));
        })
    });
}

#[cfg(feature = "rustcrypto")]
criterion_group!(
    name = rustcrypto_benches;
    config = Criterion::default();
    targets = bench_rustcrypto_sha1_small,
    bench_rustcrypto_sha256_small,
    bench_rustcrypto_sha1_large,
    bench_rustcrypto_sha256_large
);

#[cfg(feature = "boringssl")]
criterion_group!(
    name = boringssl_benches;
    config = Criterion::default();
    targets = bench_boring_sha1_small,
    bench_boring_sha256_small,
    bench_boring_sha1_large,
    bench_boring_sha256_large
);

#[cfg(all(feature = "rustcrypto", feature = "boringssl"))]
criterion_main!(rustcrypto_benches, boringssl_benches);

#[cfg(all(feature = "rustcrypto", not(feature = "boringssl")))]
criterion_main!(rustcrypto_benches);

#[cfg(all(not(feature = "rustcrypto"), feature = "boringssl"))]
criterion_main!(boringssl_benches);
