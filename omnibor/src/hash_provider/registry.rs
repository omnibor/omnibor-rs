use crate::{hash_algorithm::HashAlgorithm, hash_provider::HashProvider};
use digest::DynDigest;
use indexmap::IndexMap;
use std::{
    any::TypeId,
    ops::Not,
    sync::{Arc, LazyLock, RwLock},
};
use tracing::warn;

/// Set the active hash provider.
///
/// If not called, hash providers will cascade in a priority order, skipping
/// any which have not been enabled at compile-time with features. The order
/// is:
///
/// 1. [RustCrypto](crate::hash_provider::RustCrypto)
/// 2. [BoringSSL](crate::hash_provider::BoringSsl)
/// 3. [OpenSSL](crate::hash_provider::OpenSsl)
///
/// The type parameters are the hash algorithm used and the hash provider to
/// use for that hash algorithm.
///
/// # Example
///
/// ```
/// use omnibor::set_hash_provider;
/// use omnibor::hash_algorithm::Sha256;
/// use omnibor::hash_provider::RustCrypto;
///
/// // Set the hash provider for Sha256 to RustCrypto.
/// set_hash_provider::<Sha256, RustCrypto>();
/// ```
///
/// # Panic
///
/// Will panic if, for some reason, it is provided a type which implements
/// [`HashProvider`] but which is not present in the global hash provider
/// registry.
///
/// This should only happen if the `omnibor` maintainers have made a
/// mistake and added a new hash provider while forgetting to add that new
/// provider to the registry initialization logic.
///
/// Please [open an issue][issue] on the omnibor-rs repository if that happens.
///
/// [issue]: https://github.com/omnibor/omnibor-rs/issues/new/choose
pub fn set_hash_provider<H, P>()
where
    H: HashAlgorithm,
    P: HashProvider<H>,
{
    let ty = TypeId::of::<P>();

    if HASH_PROVIDERS.options.contains_key(&ty).not() {
        panic!("hash provider type exists but provider is not present in registry");
    }

    match HASH_PROVIDERS.active.write() {
        Ok(mut guard) => *guard = ty,
        Err(poisoned_err) => {
            warn!("hash provider registry poisoned, continuing with last-good provider");
            let mut guard = poisoned_err.into_inner();
            *guard = ty;
        }
    }
}

/// Get a `DigesterFactory` for the current hash provider.
pub(crate) fn get_hash_provider() -> DigesterFactory {
    let active = match HASH_PROVIDERS.active.read() {
        Ok(guard) => *guard,
        Err(poisoned_err) => {
            warn!("hash provider registry poisoned, continuing with last-good provider");
            *poisoned_err.into_inner()
        }
    };

    HASH_PROVIDERS.options[&active]
}

/// Global registry of hash providers, tracking all known providers and the active provider.
static HASH_PROVIDERS: LazyLock<GlobalHashProvider> = LazyLock::new(GlobalHashProvider::new);

/// Manages available and active hash providers.
struct GlobalHashProvider {
    /// The possible hash providers, initialized based on compilation features.
    options: IndexMap<TypeId, DigesterFactory>,
    /// The active hash provider, inside a RwLock since it may change.
    active: Arc<RwLock<TypeId>>,
}

impl GlobalHashProvider {
    /// Initialize the global hash provider.
    fn new() -> Self {
        let mut options = IndexMap::new();

        init_rustcrypto_providers(&mut options);
        init_boringssl_providers(&mut options);
        init_openssl_providers(&mut options);

        // SAFETY: We know at least one provider is always active.
        let active = Arc::new(RwLock::new(*options.first().unwrap().0));

        GlobalHashProvider { options, active }
    }
}

// This section uses a little trick to initialize the global provider registry.
// The specific provider types are only defined if their associated feature is
// turned on, which means they can only be referred to within functions that
// are conditional on the same feature.
//
// To deal with that, this pattern defines the same function twice, once for
// when the feature is turned on, once for when it's turned off. When it's
// turned on, the function adds the provider type to the registry. When it's
// turned off, it does nothing. This ensures we never try to name a type
// outside of a context where it exists.

#[cfg(feature = "provider-rustcrypto")]
fn init_rustcrypto_providers(options: &mut IndexMap<TypeId, DigesterFactory>) {
    options.insert(
        TypeId::of::<crate::hash_provider::RustCrypto>(),
        DigesterFactory(DigestOption::RustCrypto),
    );
}
#[cfg(not(feature = "provider-rustcrypto"))]
fn init_rustcrypto_providers(_options: &mut IndexMap<TypeId, DigesterFactory>) {
    // Intentionally blank.
}

#[cfg(feature = "provider-boringssl")]
fn init_boringssl_providers(options: &mut IndexMap<TypeId, DigesterFactory>) {
    options.insert(
        TypeId::of::<crate::hash_provider::BoringSsl>(),
        DigesterFactory {
            current_option: DigestOption::BoringSsl,
        },
    );
}
#[cfg(not(feature = "provider-boringssl"))]
fn init_boringssl_providers(_options: &mut IndexMap<TypeId, DigesterFactory>) {
    // Intentionally blank.
}

#[cfg(feature = "provider-openssl")]
fn init_openssl_providers(options: &mut IndexMap<TypeId, DigesterFactory>) {
    options.insert(
        TypeId::of::<crate::hash_provider::OpenSsl>(),
        DigesterFactory {
            current_option: DigestOption::OpenSsl,
        },
    );
}
#[cfg(not(feature = "provider-openssl"))]
fn init_openssl_providers(_options: &mut IndexMap<TypeId, DigesterFactory>) {
    // Intentionally blank.
}

/// Helper type for producing digesters.
#[derive(Debug, Copy, Clone)]
pub(crate) struct DigesterFactory(DigestOption);

impl DigesterFactory {
    /// Get the current digester.
    pub(crate) fn digester(&self) -> Box<dyn DynDigest + Send> {
        match self.0 {
            DigestOption::RustCrypto => rustcrypto_digester(),
            DigestOption::BoringSsl => boringssl_digester(),
            DigestOption::OpenSsl => openssl_digester(),
        }
    }
}

/// The options for digester.
///
/// The `#[allow(unused)]` annotation is there because the variants may be
/// unused depending on the features turned on for the crate. We don't want
/// warnings when a user doesn't turn on all providers (in fact, we generally
/// expect most users only ever want one provider).
#[allow(unused)]
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub(crate) enum DigestOption {
    RustCrypto,
    BoringSsl,
    OpenSsl,
}

// This repeats the trick used above for the `init_*` functions.

#[cfg(feature = "provider-rustcrypto")]
fn rustcrypto_digester() -> Box<dyn DynDigest + Send> {
    use crate::hash_provider::RustCrypto;
    Box::new(RustCrypto::new().digester())
}

#[cfg(not(feature = "provider-rustcrypto"))]
fn rustcrypto_digester() -> Box<dyn DynDigest + Send> {
    unimplemented!("this function is never called when the feature is off")
}

#[cfg(feature = "provider-boringssl")]
fn boringssl_digester() -> Box<dyn DynDigest + Send> {
    use crate::hash_provider::BoringSsl;
    Box::new(BoringSsl::new().digester())
}

#[cfg(not(feature = "provider-boringssl"))]
fn boringssl_digester() -> Box<dyn DynDigest + Send> {
    unimplemented!("this function is never called when the feature is off")
}

#[cfg(feature = "provider-openssl")]
fn openssl_digester() -> Box<dyn DynDigest + Send> {
    use crate::hash_provider::OpenSsl;
    Box::new(OpenSsl::new().digester())
}

#[cfg(not(feature = "provider-openssl"))]
fn openssl_digester() -> Box<dyn DynDigest + Send> {
    unimplemented!("this function is never called when the feature is off")
}
