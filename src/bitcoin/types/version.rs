use std::io::{self, BufRead, Write};

use borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
/// The transaction version.
///
/// Currently, as specified by [BIP-68], only version 1 and 2 are considered standard.
///
/// Standardness of the inner `i32` is not an invariant because you are free to create transactions
/// of any version, transactions with non-standard version numbers will not be relayed by the
/// Bitcoin network.
///
/// [BIP-68]: https://github.com/bitcoin/bips/blob/master/bip-0068.mediawiki
#[derive(
    Debug, Copy, PartialEq, Eq, Clone, Serialize, Deserialize, BorshSerialize, BorshDeserialize,
)]
pub struct Version(pub i32);

impl Version {
    /// The original Bitcoin transaction version (pre-BIP-68).
    pub const ONE: Self = Self(1);

    /// The second Bitcoin transaction version (post-BIP-68).
    pub const TWO: Self = Self(2);

    /// Serializes the version in little-endian format.
    pub fn encode<W: Write>(&self, w: &mut W) -> io::Result<()> {
        let bytes = self.0.to_le_bytes();
        // Ensure the length of the bytes is 4
        assert_eq!(bytes.len(), 4, "The length of the bytes must be 4");
        w.write_all(&bytes)
    }

    /// Deserializes the version from a buffer in little-endian format.
    pub fn decode<R: BufRead>(r: &mut R) -> io::Result<Self> {
        let mut buf = [0u8; 4];
        r.read_exact(&mut buf)?;
        Ok(Version(i32::from_le_bytes(buf)))
    }

    /// Returns the hexadecimal representation of the version.
    pub fn to_hex(&self) -> String {
        hex::encode(&self.0.to_le_bytes())
    }

    /// Serializes the version and returns the result as a Vec<u8>.
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_le_bytes().to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_version_serialization() {
        let version = Version(1);
        let mut buf = Vec::new();

        version.encode(&mut buf).unwrap();

        // Check that the serialized bytes are correct
        assert_eq!(buf, vec![1, 0, 0, 0]);

        // Check the hexadecimal representation
        assert_eq!(version.to_hex(), "01000000");
    }

    #[test]
    fn test_version_deserialization() {
        let data = vec![1, 0, 0, 0];
        let mut cursor = Cursor::new(data);
        let version = Version::decode(&mut cursor).unwrap();

        // Check that the deserialized version is correct
        assert_eq!(version, Version(1));
    }

    #[test]
    fn test_version_round_trip() {
        let version = Version(2);
        let mut buf = Vec::new();
        version.encode(&mut buf).unwrap();
        let mut cursor = Cursor::new(buf);
        let decoded_version = Version::decode(&mut cursor).unwrap();

        // Check that the version is the same after encoding and decoding
        assert_eq!(version, decoded_version);
    }

    #[test]
    fn test_version_to_vec() {
        let version = Version(1);
        let vec = version.to_vec();

        // Check that the serialized bytes are correct
        assert_eq!(vec, vec![1, 0, 0, 0]);
    }

    #[test]
    fn test_version_to_hex() {
        let version = Version(1);
        let hex = version.to_hex();

        // Check that the hexadecimal representation is correct
        assert_eq!(hex, "01000000");
    }

    #[test]
    fn test_version_invalid() {
        let version = Version(1);
        let invalid_version = Version(3);
        assert_eq!(version, invalid_version);
    }
}