use criterion::black_box;
use criterion::criterion_group;
use criterion::criterion_main;
use criterion::Criterion;
#[cfg(all(feature = "backend-boringssl", feature = "sha1"))]
use gitoid::boringssl::Sha1 as BoringSha1;
#[cfg(all(feature = "backend-boringssl", feature = "sha256"))]
use gitoid::boringssl::Sha256 as BoringSha256;
#[cfg(all(feature = "backend-openssl", feature = "sha1"))]
use gitoid::openssl::Sha1 as OpenSSLSha1;
#[cfg(all(feature = "backend-openssl", feature = "sha1"))]
use gitoid::openssl::Sha256 as OpenSSLSha256;
#[cfg(all(feature = "backend-rustcrypto", feature = "sha1"))]
use gitoid::rustcrypto::Sha1 as RustSha1;
#[cfg(all(feature = "backend-rustcrypto", feature = "sha256"))]
use gitoid::rustcrypto::Sha256 as RustSha256;
use gitoid::Blob;
use gitoid::GitOid;

#[cfg(not(any(feature = "backend-rustcrypto", feature = "backend-boringssl")))]
compile_error!(
    r#"At least one cryptography backend must be active: "backend-rustcrypto" and/or "backend-boringssl""#
);

#[cfg(feature = "backend-rustcrypto")]
fn bench_rustcrypto_sha1_small(c: &mut Criterion) {
    let name = "GitOid RustCrypto SHA-1 11B";
    let input = b"hello world";
    c.bench_function(name, |b| {
        b.iter(|| {
            let _ = GitOid::<RustSha1, Blob>::id_bytes(black_box(input));
        })
    });
}

#[cfg(feature = "backend-boringssl")]
fn bench_boring_sha1_small(c: &mut Criterion) {
    let name = "GitOid BoringSSL SHA-1 11B";
    let input = b"hello world";
    c.bench_function(name, |b| {
        b.iter(|| {
            let _ = GitOid::<BoringSha1, Blob>::id_bytes(black_box(input));
        })
    });
}

#[cfg(feature = "backend-openssl")]
fn bench_openssl_sha1_small(c: &mut Criterion) {
    let name = "GitOid OpenSSL SHA-1 11B";
    let input = b"hello world";
    c.bench_function(name, |b| {
        b.iter(|| {
            let _ = GitOid::<OpenSSLSha1, Blob>::id_bytes(black_box(input));
        })
    });
}

#[cfg(feature = "backend-rustcrypto")]
fn bench_rustcrypto_sha256_small(c: &mut Criterion) {
    let name = "GitOid RustCrypto SHA-256 11B";
    let input = b"hello world";
    c.bench_function(name, |b| {
        b.iter(|| {
            let _ = GitOid::<RustSha256, Blob>::id_bytes(black_box(input));
        })
    });
}

#[cfg(feature = "backend-boringssl")]
fn bench_boring_sha256_small(c: &mut Criterion) {
    let name = "GitOid BoringSSL SHA-256 11B";
    let input = b"hello world";
    c.bench_function(name, |b| {
        b.iter(|| {
            let _ = GitOid::<BoringSha256, Blob>::id_bytes(black_box(input));
        })
    });
}

#[cfg(feature = "backend-openssl")]
fn bench_openssl_sha256_small(c: &mut Criterion) {
    let name = "GitOid OpenSSL SHA-256 11B";
    let input = b"hello world";
    c.bench_function(name, |b| {
        b.iter(|| {
            let _ = GitOid::<OpenSSLSha256, Blob>::id_bytes(black_box(input));
        })
    });
}

#[cfg(feature = "backend-rustcrypto")]
fn bench_rustcrypto_sha1_large(c: &mut Criterion) {
    let name = "GitOid RustCrypto SHA-1 100MB";
    let input = &[0; 1024 * 1024 * 100]; // 100 MB
    c.bench_function(name, |b| {
        b.iter(|| {
            let _ = GitOid::<RustSha1, Blob>::id_bytes(black_box(input));
        })
    });
}

#[cfg(feature = "backend-boringssl")]
fn bench_boring_sha1_large(c: &mut Criterion) {
    let name = "GitOid BoringSSL SHA-1 100MB";
    let input = &[0; 1024 * 1024 * 100]; // 100 MB
    c.bench_function(name, |b| {
        b.iter(|| {
            let _ = GitOid::<BoringSha1, Blob>::id_bytes(black_box(input));
        })
    });
}

#[cfg(feature = "backend-openssl")]
fn bench_openssl_sha1_large(c: &mut Criterion) {
    let name = "GitOid OpenSSL SHA-1 100MB";
    let input = &[0; 1024 * 1024 * 100]; // 100 MB
    c.bench_function(name, |b| {
        b.iter(|| {
            let _ = GitOid::<OpenSSLSha1, Blob>::id_bytes(black_box(input));
        })
    });
}

#[cfg(feature = "backend-rustcrypto")]
fn bench_rustcrypto_sha256_large(c: &mut Criterion) {
    let name = "GitOid RustCrypto SHA-256 100MB";
    let input = &[0; 1024 * 1024 * 100]; // 100 MB
    c.bench_function(name, |b| {
        b.iter(|| {
            let _ = GitOid::<RustSha256, Blob>::id_bytes(black_box(input));
        })
    });
}

#[cfg(feature = "backend-boringssl")]
fn bench_boring_sha256_large(c: &mut Criterion) {
    let name = "GitOid BoringSSL SHA-256 100MB";
    let input = &[0; 1024 * 1024 * 100]; // 100 MB
    c.bench_function(name, |b| {
        b.iter(|| {
            let _ = GitOid::<BoringSha256, Blob>::id_bytes(black_box(input));
        })
    });
}

#[cfg(feature = "backend-openssl")]
fn bench_openssl_sha256_large(c: &mut Criterion) {
    let name = "GitOid OpenSSL SHA-256 100MB";
    let input = &[0; 1024 * 1024 * 100]; // 100 MB
    c.bench_function(name, |b| {
        b.iter(|| {
            let _ = GitOid::<OpenSSLSha256, Blob>::id_bytes(black_box(input));
        })
    });
}

#[cfg(feature = "backend-rustcrypto")]
criterion_group!(
    name = rustcrypto_benches;
    config = Criterion::default();
    targets = bench_rustcrypto_sha1_small,
    bench_rustcrypto_sha256_small,
    bench_rustcrypto_sha1_large,
    bench_rustcrypto_sha256_large
);

#[cfg(feature = "backend-boringssl")]
criterion_group!(
    name = boringssl_benches;
    config = Criterion::default();
    targets = bench_boring_sha1_small,
    bench_boring_sha256_small,
    bench_boring_sha1_large,
    bench_boring_sha256_large
);

#[cfg(feature = "backend-openssl")]
criterion_group!(
    name = openssl_benches;
    config = Criterion::default();
    targets = bench_openssl_sha1_small,
    bench_openssl_sha256_small,
    bench_openssl_sha1_large,
    bench_openssl_sha256_large
);

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
criterion_main!(boringssl_benches, openssl_benches());

#[cfg(all(
    feature = "backend-rustcrypto",
    not(feature = "backend-boringssl"),
    feature = "backend-openssl"
))]
criterion_main!(rustcrypto_benches, openssl_benches);

#[cfg(all(
    feature = "backend-rustcrypto",
    not(feature = "backend-boringssl"),
    not(feature = "backend-openssl")
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
