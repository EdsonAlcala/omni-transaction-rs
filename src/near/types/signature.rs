use borsh::{BorshDeserialize, BorshSerialize};
use bs58;
use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::Debug;

use crate::constants::{COMPONENT_SIZE, SECP256K1_SIGNATURE_LENGTH};

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq, Eq, JsonSchema)]
#[serde(crate = "near_sdk::serde")]
pub enum Signature {
    ED25519(ED25519Signature),
    SECP256K1(Secp256K1Signature),
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq, Eq, JsonSchema)]
pub struct ED25519Signature {
    pub r: ComponentBytes,
    pub s: ComponentBytes,
}

/// Size of an `R` or `s` component of an Ed25519 signature when serialized as bytes.
pub type ComponentBytes = [u8; COMPONENT_SIZE];

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq, Eq)]
pub struct Secp256K1Signature(pub [u8; SECP256K1_SIGNATURE_LENGTH]);

impl JsonSchema for Secp256K1Signature {
    fn schema_name() -> String {
        "Secp256K1Signature".to_owned()
    }

    fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        <String>::json_schema(gen)
    }
}

impl Serialize for Signature {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::ED25519(sig) => {
                let mut bytes = Vec::with_capacity(COMPONENT_SIZE * 2);
                bytes.extend_from_slice(&sig.r);
                bytes.extend_from_slice(&sig.s);

                let encoded = bs58::encode(&bytes).into_string();
                serializer.serialize_str(&format!("ed25519:{encoded}"))
            }
            Self::SECP256K1(sig) => {
                let encoded = bs58::encode(&sig.0).into_string();
                serializer.serialize_str(&format!("secp256k1:{encoded}"))
            }
        }
    }
}

impl<'de> Deserialize<'de> for Signature {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        let (key_type, sig_data) = s.split_at(
            s.find(':')
                .ok_or_else(|| serde::de::Error::custom("Invalid signature format"))?,
        );
        let sig_data = &sig_data[1..]; // Skip the colon

        match key_type {
            "ed25519" => {
                let bytes = bs58::decode(sig_data)
                    .into_vec()
                    .map_err(serde::de::Error::custom)?;

                let signature = ED25519Signature {
                    r: bytes[0..COMPONENT_SIZE]
                        .try_into()
                        .map_err(serde::de::Error::custom)?,
                    s: bytes[COMPONENT_SIZE..]
                        .try_into()
                        .map_err(serde::de::Error::custom)?,
                };
                Ok(Self::ED25519(signature))
            }
            "secp256k1" => {
                let bytes = bs58::decode(sig_data)
                    .into_vec()
                    .map_err(serde::de::Error::custom)?;

                if bytes.len() != SECP256K1_SIGNATURE_LENGTH {
                    return Err(serde::de::Error::custom(
                        "Invalid SECP256K1 signature length",
                    ));
                }

                let mut array = [0u8; SECP256K1_SIGNATURE_LENGTH];
                array.copy_from_slice(&bytes);
                Ok(Self::SECP256K1(Secp256K1Signature(array)))
            }
            _ => Err(serde::de::Error::custom("Unknown key type")),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::near::utils::SignatureStrExt;

    use super::*;
    use serde_json;

    #[test]
    fn test_deserialize_ed25519_signature() {
        let serialized = "\"ed25519:3s1dvZdQtcAjBksMHFrysqvF63wnyMHPA4owNQmCJZ2EBakZEKdtMsLqrHdKWQjJbSRN6kRknN2WdwSBLWGCokXj\"";
        let deserialized: Signature = serde_json::from_str(serialized).unwrap();

        let decoded = "ed25519:3s1dvZdQtcAjBksMHFrysqvF63wnyMHPA4owNQmCJZ2EBakZEKdtMsLqrHdKWQjJbSRN6kRknN2WdwSBLWGCokXj".to_signature_as_bytes().unwrap();

        let expected = Signature::ED25519(ED25519Signature {
            r: decoded[0..COMPONENT_SIZE].try_into().unwrap(),
            s: decoded[COMPONENT_SIZE..].try_into().unwrap(),
        });

        assert_eq!(deserialized, expected);
    }

    #[test]
    fn test_deserialize_secp256k1_signature() {
        let serialized = "\"secp256k1:5N5CB9H1dmB9yraLGCo4ZCQTcF24zj4v2NT14MHdH3aVhRoRXrX3AhprHr2w6iXNBZDmjMS1Ntzjzq8Bv6iBvwth6\"";
        let deserialized: Signature = serde_json::from_str(serialized).unwrap();

        let decoded = "secp256k1:5N5CB9H1dmB9yraLGCo4ZCQTcF24zj4v2NT14MHdH3aVhRoRXrX3AhprHr2w6iXNBZDmjMS1Ntzjzq8Bv6iBvwth6".to_signature_as_bytes().unwrap();

        let expected = Signature::SECP256K1(Secp256K1Signature(decoded.try_into().unwrap()));

        assert_eq!(deserialized, expected);
    }

    #[test]
    fn test_deserialize_with_invalid_data() {
        let invalid = "\"secp256k1:2xVqteU8PWhadHTv99TGh3bSf\"";

        assert!(serde_json::from_str::<Signature>(invalid).is_err());
    }

    #[test]
    fn test_serialize_ed25519_signature() {
        // Decode the base58 signature to get the components r and s
        let decoded = "ed25519:3s1dvZdQtcAjBksMHFrysqvF63wnyMHPA4owNQmCJZ2EBakZEKdtMsLqrHdKWQjJbSRN6kRknN2WdwSBLWGCokXj".to_signature_as_bytes().unwrap();

        let signature = Signature::ED25519(ED25519Signature {
            r: decoded[0..COMPONENT_SIZE].try_into().unwrap(),
            s: decoded[COMPONENT_SIZE..].try_into().unwrap(),
        });

        let serialized = serde_json::to_string(&signature).unwrap();
        let expected = "\"ed25519:3s1dvZdQtcAjBksMHFrysqvF63wnyMHPA4owNQmCJZ2EBakZEKdtMsLqrHdKWQjJbSRN6kRknN2WdwSBLWGCokXj\"";

        assert_eq!(serialized, expected);
    }

    #[test]
    fn test_serialize_secp256k1_signature() {
        // Decode the base58 signature to get the array of bytes
        let decoded = "secp256k1:5N5CB9H1dmB9yraLGCo4ZCQTcF24zj4v2NT14MHdH3aVhRoRXrX3AhprHr2w6iXNBZDmjMS1Ntzjzq8Bv6iBvwth6".to_signature_as_bytes().unwrap();

        let signature = Signature::SECP256K1(Secp256K1Signature(decoded.try_into().unwrap()));
        let serialized = serde_json::to_string(&signature).unwrap();
        let expected = "\"secp256k1:5N5CB9H1dmB9yraLGCo4ZCQTcF24zj4v2NT14MHdH3aVhRoRXrX3AhprHr2w6iXNBZDmjMS1Ntzjzq8Bv6iBvwth6\"";

        assert_eq!(serialized, expected);
    }

    #[test]
    fn test_borsh_serialize_deserialize_ed25519() {
        let decoded = "ed25519:3s1dvZdQtcAjBksMHFrysqvF63wnyMHPA4owNQmCJZ2EBakZEKdtMsLqrHdKWQjJbSRN6kRknN2WdwSBLWGCokXj".to_signature_as_bytes().unwrap();

        let signature = Signature::ED25519(ED25519Signature {
            r: decoded[0..COMPONENT_SIZE].try_into().unwrap(),
            s: decoded[COMPONENT_SIZE..].try_into().unwrap(),
        });

        let serialized = borsh::to_vec(&signature).unwrap();
        let deserialized: Signature = borsh::BorshDeserialize::try_from_slice(&serialized).unwrap();

        assert_eq!(signature, deserialized);
    }

    #[test]
    fn test_borsh_serialize_deserialize_secp256k1() {
        let decoded = "secp256k1:5N5CB9H1dmB9yraLGCo4ZCQTcF24zj4v2NT14MHdH3aVhRoRXrX3AhprHr2w6iXNBZDmjMS1Ntzjzq8Bv6iBvwth6".to_signature_as_bytes().unwrap();

        let signature = Signature::SECP256K1(Secp256K1Signature(decoded.try_into().unwrap()));

        let serialized = borsh::to_vec(&signature).unwrap();
        let deserialized: Signature = borsh::BorshDeserialize::try_from_slice(&serialized).unwrap();

        assert_eq!(signature, deserialized);
    }
}
