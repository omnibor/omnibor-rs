use crate::{error::HashProviderError, hash_algorithm::HashAlgorithm, hash_provider::HashProvider};
use digest::DynDigest;
use indexmap::IndexMap;
use std::{any::TypeId, ops::Not, sync::LazyLock};
use tokio::sync::Mutex;

/// Set the active hash provider.
pub fn set_hash_provider<H, P>() -> Result<(), HashProviderError>
where
    H: HashAlgorithm,
    P: HashProvider<H>,
{
    let ty = TypeId::of::<P>();

    if HASH_PROVIDERS.options.contains_key(&ty).not() {
        return Err(HashProviderError::InvalidHashProvider);
    }

    *HASH_PROVIDERS.active.blocking_lock() = ty;
    Ok(())
}

/// Set the active hash provider asynchronously.
pub async fn set_hash_provider_async<H, P>() -> Result<(), HashProviderError>
where
    H: HashAlgorithm,
    P: HashProvider<H>,
{
    let ty = TypeId::of::<P>();

    if HASH_PROVIDERS.options.contains_key(&ty).not() {
        return Err(HashProviderError::InvalidHashProvider);
    }

    *HASH_PROVIDERS.active.lock().await = ty;
    Ok(())
}

pub(crate) fn get_hash_provider() -> DigesterFactory {
    let active = *HASH_PROVIDERS.active.blocking_lock();
    HASH_PROVIDERS.options[&active]
}

pub(crate) async fn get_hash_provider_async() -> AsyncDigesterFactory {
    let active = *HASH_PROVIDERS.active.lock().await;
    AsyncDigesterFactory {
        factory: HASH_PROVIDERS.options[&active],
    }
}

static HASH_PROVIDERS: LazyLock<GlobalHashProvider> = LazyLock::new(GlobalHashProvider::new);

/// Manages available and active hash providers.
struct GlobalHashProvider {
    /// The possible hash providers, initialized based on compilation features.
    options: IndexMap<TypeId, DigesterFactory>,
    /// The active hash provider, inside a Mutex since it may change.
    active: Mutex<TypeId>,
}

impl GlobalHashProvider {
    fn new() -> Self {
        let mut options = IndexMap::new();

        init_rustcrypto_providers(&mut options);
        init_boringssl_providers(&mut options);
        init_openssl_providers(&mut options);

        // SAFETY: We know at least one provider is always active.
        let active = Mutex::new(*options.first().unwrap().0);

        GlobalHashProvider { options, active }
    }
}

#[cfg(feature = "provider-rustcrypto")]
fn init_rustcrypto_providers(options: &mut IndexMap<TypeId, DigesterFactory>) {
    options.insert(
        TypeId::of::<crate::hash_provider::RustCrypto>(),
        DigesterFactory {
            current_option: DigestOption::RustCrypto,
        },
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

#[derive(Debug, Copy, Clone)]
pub(crate) struct DigesterFactory {
    current_option: DigestOption,
}

impl DigesterFactory {
    pub(crate) fn digester(&self) -> Box<dyn DynDigest + Send> {
        match self.current_option {
            DigestOption::RustCrypto => rustcrypto_digester(),
            DigestOption::BoringSsl => boringssl_digester(),
            DigestOption::OpenSsl => openssl_digester(),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub(crate) struct AsyncDigesterFactory {
    factory: DigesterFactory,
}

impl AsyncDigesterFactory {
    pub(crate) fn digester(&self) -> Box<dyn DynDigest + Send> {
        self.factory.digester()
    }
}

#[allow(unused)]
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub(crate) enum DigestOption {
    RustCrypto,
    BoringSsl,
    OpenSsl,
}

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
