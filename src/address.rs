use {
    crate::{format::SolanaFormat, public_key::SolanaPublicKey},
    anychain_core::{Address, AddressError, PublicKey, PublicKeyError},
    core::{
        fmt::{Display, Formatter, Result as FmtResult},
        str::FromStr,
    },
    curve25519_dalek::Scalar,
    ed25519_dalek::PUBLIC_KEY_LENGTH,
    solana_sdk::pubkey::Pubkey,
    spl_associated_token_account::get_associated_token_address,
};

/// Represents a Solana address
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SolanaAddress(pub String);

impl SolanaAddress {
    pub fn associated_token_address(&self, token: String) -> Result<String, AddressError> {
        let address =
            Pubkey::from_str(&self.0).map_err(|e| AddressError::Message(format!("{e}")))?;
        let token = Pubkey::from_str(&token).map_err(|e| AddressError::Message(format!("{e}")))?;
        let associated_token_address = get_associated_token_address(&address, &token);
        Ok(associated_token_address.to_string())
    }
}

impl Address for SolanaAddress {
    type SecretKey = Scalar;
    type Format = SolanaFormat;
    type PublicKey = SolanaPublicKey;

    fn from_secret_key(
        secret_key: &Self::SecretKey,
        format: &Self::Format,
    ) -> Result<Self, AddressError> {
        Self::PublicKey::from_secret_key(secret_key).to_address(format)
    }

    fn from_public_key(
        public_key: &Self::PublicKey,
        _: &Self::Format,
    ) -> Result<Self, AddressError> {
        let address = bs58::encode(public_key.0.to_bytes()).into_string();
        Ok(Self(address))
    }

    fn is_valid(address: &str) -> bool {
        Self::from_str(address).is_ok()
    }
}

impl FromStr for SolanaAddress {
    type Err = AddressError;

    fn from_str(addr: &str) -> Result<Self, Self::Err> {
        // Check if the address is valid
        if addr.len() > crate::public_key::MAX_BASE58_LEN {
            return Err(AddressError::InvalidCharacterLength(addr.len()));
        }
        let pubkey_vec = bs58::decode(addr)
            .into_vec()
            .map_err(|error| PublicKeyError::Crate("base58", format!("{error:?}")))?;
        if pubkey_vec.len() != PUBLIC_KEY_LENGTH {
            return Err(AddressError::InvalidAddress(addr.to_string()));
        }
        Ok(Self(addr.to_string()))
    }
}

impl Display for SolanaAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{KEYPAIR_LENGTH, PUBLIC_KEY_LENGTH, SECRET_KEY_LENGTH};

    #[test]
    fn test_address_alice() {
        let keypair_bytes: [u8; KEYPAIR_LENGTH] = [
            41, 196, 252, 146, 80, 100, 13, 46, 69, 89, 172, 157, 224, 135, 23, 62, 54, 65, 52, 68,
            14, 50, 112, 112, 156, 210, 24, 236, 139, 169, 38, 63, 205, 66, 112, 255, 116, 177, 79,
            182, 192, 20, 240, 193, 219, 162, 23, 149, 26, 247, 181, 186, 145, 168, 26, 232, 228,
            76, 102, 109, 64, 189, 172, 44,
        ];

        let mut secret_bytes: [u8; PUBLIC_KEY_LENGTH] = [0u8; SECRET_KEY_LENGTH];
        secret_bytes.copy_from_slice(&keypair_bytes[0..SECRET_KEY_LENGTH]);

        let secret_key = Scalar::from_bytes_mod_order(secret_bytes);

        let address =
            SolanaAddress::from_secret_key(&secret_key, &SolanaFormat::default()).unwrap();
        assert_eq!(
            "8tR45MbTcEq1W4dMXnwe7KW7xqykNxnyoBQoASMtqHK",
            address.to_string()
        );
    }

    #[test]
    fn test_address_bob() {
        let keypair_bytes: [u8; KEYPAIR_LENGTH] = [
            47, 232, 53, 167, 54, 186, 162, 109, 156, 250, 166, 187, 29, 118, 132, 137, 28, 228,
            202, 245, 100, 119, 252, 44, 3, 55, 22, 129, 80, 11, 154, 149, 178, 218, 84, 101, 24,
            203, 245, 149, 168, 220, 195, 44, 240, 213, 89, 146, 82, 159, 117, 129, 133, 128, 7,
            99, 136, 179, 15, 161, 42, 132, 31, 41,
        ];

        let mut secret_bytes: [u8; PUBLIC_KEY_LENGTH] = [0u8; SECRET_KEY_LENGTH];
        secret_bytes.copy_from_slice(&keypair_bytes[0..SECRET_KEY_LENGTH]);
        let secret_key = Scalar::from_bytes_mod_order(secret_bytes);

        let address =
            SolanaAddress::from_secret_key(&secret_key, &SolanaFormat::default()).unwrap();

        assert_eq!(
            "DPCG5xKuxK3NCL8FTn8h2vFharp9MbYmUZeSA2eLx4RG",
            address.to_string()
        );
    }

    #[test]
    fn test_is_valid_address() {
        assert!(SolanaAddress::is_valid(
            "6p6xgHyF7AeE6TZkSmFsko444wqoP15icUSqi2jfGiPN"
        ));
        assert!(SolanaAddress::is_valid(
            "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB"
        ));
    }
}
