use criterion::black_box;
use criterion::criterion_group;
use criterion::criterion_main;
use criterion::Criterion;
#[cfg(all(feature = "rustcrypto", feature = "sha1"))]
use gitoid::rustcrypto::Sha1 as RustSha1;
#[cfg(all(feature = "rustcrypto", feature = "sha256"))]
use gitoid::rustcrypto::Sha256 as RustSha256;
use gitoid::Blob;
#[cfg(all(feature = "boringssl", feature = "sha1"))]
use gitoid::boringssl::Sha1 as BoringSha1;
#[cfg(all(feature = "boringssl", feature = "sha256"))]
use gitoid::boringssl::Sha256 as BoringSha256;
use gitoid::GitOid;
#[cfg(all(feature = "openssl", feature = "sha1"))]
use gitoid::openssl::Sha256 as OpenSSLSha256;
#[cfg(all(feature = "openssl", feature = "sha1"))]
use gitoid::openssl::Sha1 as OpenSSLSha1;

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

#[cfg(feature = "openssl")]
fn bench_openssl_sha1_small(c: &mut Criterion) {
    let name = "GitOid OpenSSL SHA-1 11B";
    let input = b"hello world";
    c.bench_function(name, |b| {
        b.iter(|| {
            let _ = GitOid::<OpenSSLSha1, Blob>::id_bytes(black_box(input));
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

#[cfg(feature = "openssl")]
fn bench_openssl_sha256_small(c: &mut Criterion) {
    let name = "GitOid OpenSSL SHA-256 11B";
    let input = b"hello world";
    c.bench_function(name, |b| {
        b.iter(|| {
            let _ = GitOid::<OpenSSLSha256, Blob>::id_bytes(black_box(input));
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

#[cfg(feature = "openssl")]
fn bench_openssl_sha1_large(c: &mut Criterion) {
    let name = "GitOid OpenSSL SHA-1 100MB";
    let input = &[0; 1024 * 1024 * 100]; // 100 MB
    c.bench_function(name, |b| {
        b.iter(|| {
            let _ = GitOid::<OpenSSLSha1, Blob>::id_bytes(black_box(input));
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

#[cfg(feature = "openssl")]
fn bench_openssl_sha256_large(c: &mut Criterion) {
    let name = "GitOid OpenSSL SHA-256 100MB";
    let input = &[0; 1024 * 1024 * 100]; // 100 MB
    c.bench_function(name, |b| {
        b.iter(|| {
            let _ = GitOid::<OpenSSLSha256, Blob>::id_bytes(black_box(input));
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

#[cfg(feature = "openssl")]
criterion_group!(
    name = openssl_benches;
    config = Criterion::default();
    targets = bench_openssl_sha1_small,
    bench_openssl_sha256_small,
    bench_openssl_sha1_large,
    bench_openssl_sha256_large
);

#[cfg(all(feature = "rustcrypto", feature = "boringssl", feature = "openssl"))]
criterion_main!(rustcrypto_benches, boringssl_benches, openssl_benches);

#[cfg(all(feature = "rustcrypto", feature = "boringssl", not(feature= "openssl")))]
criterion_main!(rustcrypto_benches, boringssl_benches);

#[cfg(all(not(feature = "rustcrypto"), feature = "boringssl", feature= "openssl"))]
criterion_main!(boringssl_benches, openssl_benches());

#[cfg(all(feature = "rustcrypto", not(feature = "boringssl"), feature= "openssl"))]
criterion_main!(rustcrypto_benches, openssl_benches);

#[cfg(all(feature = "rustcrypto", not(feature = "boringssl"), not(feature= "openssl")))]
criterion_main!(rustcrypto_benches);

#[cfg(all(not(feature = "rustcrypto"), feature = "boringssl", not(feature = "openssl")))]
criterion_main!(boringssl_benches);

#[cfg(all(not(feature = "rustcrypto"), not(feature = "boringssl"), feature = "openssl"))]
criterion_main!(openssl_benches);
