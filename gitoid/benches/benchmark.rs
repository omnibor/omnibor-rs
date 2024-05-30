use criterion::{black_box, criterion_group, criterion_main, Criterion};
use gitoid::{GitOid, Sha1, Sha256, Blob};

#[cfg(not(feature = "boring"))]
fn bench_rustcrypto_sha1_small(c: &mut Criterion) {
    bench_sha1_small(c, "GitOid RustCrypto SHA-1 11B");
}
#[cfg(feature = "boring")]
fn bench_boring_sha1_small(c: &mut Criterion) {
    bench_sha1_small(c, "GitOid BoringSSL SHA-1 11B");
}

fn bench_sha1_small(c: &mut Criterion, name: &str) {
    let input = b"hello world";
    c.bench_function(name, |b| {
        b.iter(|| {
            let _ = GitOid::<Sha1, Blob>::id_bytes(black_box(input));
        })
    });
}

#[cfg(not(feature = "boring"))]
fn bench_rustcrypto_sha256_small(c: &mut Criterion) {
    bench_sha256_small(c, "GitOid RustCrypto SHA-256 11B");
}
#[cfg(feature = "boring")]
fn bench_boring_sha256_small(c: &mut Criterion) {
    bench_sha256_small(c, "GitOid BoringSSL SHA-256 11B");
}
fn bench_sha256_small(c: &mut Criterion, name: &str) {
    let input = b"hello world";
    c.bench_function(name, |b| {
        b.iter(|| {
            let _ = GitOid::<Sha256, Blob>::id_bytes(black_box(input));
        })
    });
}

#[cfg(not(feature = "boring"))]
fn bench_rustcrypto_sha1_large(c: &mut Criterion) {
    bench_sha1_large(c, "GitOid RustCrypto SHA-1 100MB");
}
#[cfg(feature = "boring")]
fn bench_boring_sha1_large(c: &mut Criterion) {
    bench_sha1_large(c, "GitOid BoringSSL SHA-1 100MB");
}

fn bench_sha1_large(c: &mut Criterion, name: &str) {
    let input = &[0; 1024*1024*100]; // 100 MB
    c.bench_function(name, |b| {
        b.iter(|| {
            let _ = GitOid::<Sha1, Blob>::id_bytes(black_box(input));
        })
    });
}

#[cfg(not(feature = "boring"))]
fn bench_rustcrypto_sha256_large(c: &mut Criterion) {
    bench_sha256_large(c, "GitOid RustCrypto SHA-256 100MB");
}
#[cfg(feature = "boring")]
fn bench_boring_sha256_large(c: &mut Criterion) {
    bench_sha256_large(c, "GitOid BoringSSL SHA-256 100MB");
}

fn bench_sha256_large(c: &mut Criterion, name: &str) {
    let input = &[0; 1024*1024*100]; // 100 MB
    c.bench_function(name, |b| {
        b.iter(|| {
            let _ = GitOid::<Sha256, Blob>::id_bytes(black_box(input));
        })
    });
}

#[cfg(not(feature = "boring"))]
criterion_group!(benches, bench_rustcrypto_sha1_small, bench_rustcrypto_sha256_small, bench_rustcrypto_sha1_large, bench_rustcrypto_sha256_large);
#[cfg(feature = "boring")]
criterion_group!(benches, bench_boring_sha1_small, bench_boring_sha256_small, bench_boring_sha1_large, bench_boring_sha256_large);
criterion_main!(benches);
