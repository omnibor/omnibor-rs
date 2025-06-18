use crate::{error::InputManifestError, util::clone_as_boxstr::CloneAsBoxstr};
use rand::prelude::*;
use std::{fmt::Display, ops::Not as _, str::FromStr};

/// The size in bytes of a BuildId.
const BUILD_ID_LEN: usize = 16;

/// The "reverse hex" character set.
const REVERSE_HEX_CHARS: &[u8; 16] = b"zyxwvutsrqponmlk";

/// A unique ID identifying a build.
#[derive(Debug)]
pub struct BuildId {
    /// The actual bytes.
    data: Vec<u8>,
}

impl BuildId {
    /// Construct a new random BuildId.
    pub fn new() -> Self {
        let mut rng = rand::rng();

        BuildId {
            data: (0..BUILD_ID_LEN).map(|_| rng.random::<u8>()).collect(),
        }
    }

    /// Encode the BuildId as "reverse hex".
    ///
    /// Adapted from Jujutsu, licensed under the Apache 2.0 license.
    ///
    /// <https://github.com/jj-vcs/jj/blob/c43ca3c07beb81447e3401eef237604accbc4dd5/lib/src/hex_util.rs#L40-L48>
    pub fn as_reverse_hex(&self) -> String {
        let chars = REVERSE_HEX_CHARS;

        let encoded = self
            .data
            .iter()
            .flat_map(|b| [chars[usize::from(b >> 4)], chars[usize::from(b & 0xf)]])
            .collect();

        // PANIC SAFETY: We know all characters are valid UTF-8, so the unwrap won't panic.
        String::from_utf8(encoded).unwrap()
    }
}

/// Convert a reverse-hex digit to a forward-hex digit.
///
/// Adapted from Jujutsu, licensed under the Apache 2.0 license.
///
/// <https://github.com/jj-vcs/jj/blob/c43ca3c07beb81447e3401eef237604accbc4dd5/lib/src/hex_util.rs#L40-L48>
fn to_forward_hex_digit(b: u8) -> Option<u8> {
    let value = match b {
        b'k'..=b'z' => b'z' - b,
        b'K'..=b'Z' => b'Z' - b,
        _ => return None,
    };
    if value < 10 {
        Some(b'0' + value)
    } else {
        Some(b'a' + value - 10)
    }
}

/// Convert a slice of reverse-hex digits to a string of forward-hex digits.
///
/// Adapted from Jujutsu, licensed under the Apache 2.0 license.
///
/// <https://github.com/jj-vcs/jj/blob/c43ca3c07beb81447e3401eef237604accbc4dd5/lib/src/hex_util.rs#L40-L48>
fn to_forward_hex(reverse_hex: impl AsRef<[u8]>) -> Option<String> {
    reverse_hex
        .as_ref()
        .iter()
        .map(|b| to_forward_hex_digit(*b).map(char::from))
        .collect()
}

impl Display for BuildId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_reverse_hex())
    }
}

impl FromStr for BuildId {
    type Err = InputManifestError;

    fn from_str(reverse_hex: &str) -> Result<Self, Self::Err> {
        if reverse_hex
            .bytes()
            .all(|b| REVERSE_HEX_CHARS.contains(&b))
            .not()
        {
            return Err(InputManifestError::InvalidBuildId(
                reverse_hex.clone_as_boxstr(),
            ));
        }

        let forward_string = to_forward_hex(&reverse_hex)
            .ok_or_else(|| InputManifestError::InvalidBuildId(reverse_hex.clone_as_boxstr()))?;

        let build_id = BuildId {
            // PANIC SAFETY: We know this is a valid hex string.
            data: hex::decode(forward_string).unwrap(),
        };

        Ok(build_id)
    }
}
