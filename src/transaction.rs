use {
    crate::{SolanaAddress, SolanaFormat, SolanaPublicKey},
    anychain_core::{Transaction, TransactionError, TransactionId},
    core::str::FromStr,
    solana_sdk::{
        hash::Hash as BlockHash, pubkey::Pubkey as RawSolanaPubkey, signature::Signer,
        signer::keypair::Keypair, system_instruction,
        transaction::Transaction as RawSolanaTransaction,
    },
    std::fmt::Display,
};

pub struct SolanaTransactionParameters {
    pub keypair: Keypair,
    pub to: SolanaAddress,
    pub lamports: u64,
    pub block_hash: BlockHash,
}

impl Clone for SolanaTransactionParameters {
    fn clone(&self) -> Self {
        Self {
            keypair: self.keypair.insecure_clone(),
            to: self.to.clone(),
            lamports: self.lamports,
            block_hash: self.block_hash,
        }
    }
}

impl SolanaTransactionParameters {
    pub fn new(keypair: Keypair, to: SolanaAddress, lamports: u64, block_hash: BlockHash) -> Self {
        Self {
            keypair,
            to,
            lamports,
            block_hash,
        }
    }
}

#[derive(Clone)]
pub struct SolanaTransaction {
    pub params: SolanaTransactionParameters,
    pub transaction: RawSolanaTransaction,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct SolanaTransactionId {
    txid: Vec<u8>,
}

impl TransactionId for SolanaTransactionId {}

impl Display for SolanaTransactionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:?}", self.txid)
    }
}

impl Transaction for SolanaTransaction {
    type Address = SolanaAddress;
    type Format = SolanaFormat;
    type PublicKey = SolanaPublicKey;
    type TransactionId = SolanaTransactionId;
    type TransactionParameters = SolanaTransactionParameters;

    fn new(params: &Self::TransactionParameters) -> Result<Self, TransactionError> {
        let SolanaTransactionParameters {
            keypair,
            to,
            lamports,
            block_hash: _,
        } = params;

        let bob_pubkey = RawSolanaPubkey::from_str(to.0.as_str())
            .map_err(|_| TransactionError::Message("Failed to parse Solana address".to_string()))?;
        let transfer_instruction =
            system_instruction::transfer(&keypair.pubkey(), &bob_pubkey, *lamports);

        // Create a transaction
        let transaction =
            RawSolanaTransaction::new_with_payer(&[transfer_instruction], Some(&keypair.pubkey()));

        Ok(Self {
            params: params.clone(),
            transaction,
        })
    }

    fn sign(&mut self, _signature: Vec<u8>, _recid: u8) -> Result<Vec<u8>, TransactionError> {
        let keypair = &self.params.keypair;
        let block_hash = self.params.block_hash;

        self.transaction.sign(&[keypair], block_hash);

        let signature = self.transaction.signatures[0].as_ref().to_vec();
        Ok(signature)
    }

    fn from_bytes(_tx: &[u8]) -> Result<Self, TransactionError> {
        todo!()
    }

    fn to_bytes(&self) -> Result<Vec<u8>, TransactionError> {
        let message_data = self.transaction.message_data();
        Ok(message_data)
    }

    fn to_transaction_id(&self) -> Result<Self::TransactionId, TransactionError> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::str::FromStr;
    use ed25519_dalek::KEYPAIR_LENGTH;
    use solana_sdk::{hash::Hash, signer::keypair::Keypair};

    // const ALICE_ADDRESS: &str = "EpFLfuH524fk9QP9i9uL9AHtX6smBaxaMHwek9T11nK5";
    const BOB_ADDRESS: &str = "D3AfQC64W8xCqwH1y94dQY4JLG6HQx6uLoHk9V6qqAKr";

    const BLOCK_HASH: &str = "3DaLmounsHb3nzDfTEDdhF2rG2UL7CiwPGXXgFzDkpoy";

    #[test]
    fn test_blockhash() {
        let hash_res = Hash::from_str(BLOCK_HASH);
        assert!(hash_res.is_ok());
    }

    #[test]
    fn test_tx_generation_one_one_transfer() {
        let keypair_bytes: [u8; KEYPAIR_LENGTH] = [
            41, 196, 252, 146, 80, 100, 13, 46, 69, 89, 172, 157, 224, 135, 23, 62, 54, 65, 52, 68,
            14, 50, 112, 112, 156, 210, 24, 236, 139, 169, 38, 63, 205, 66, 112, 255, 116, 177, 79,
            182, 192, 20, 240, 193, 219, 162, 23, 149, 26, 247, 181, 186, 145, 168, 26, 232, 228,
            76, 102, 109, 64, 189, 172, 44,
        ];

        // let address_alice = SolanaAddress::from_str(ALICE_ADDRESS).unwrap();
        let address_bob = SolanaAddress::from_str(BOB_ADDRESS).unwrap();
        let lamports = 100;
        let keypair_alice = Keypair::from_bytes(&keypair_bytes).unwrap();
        let block_hash = solana_sdk::hash::Hash::from_str(BLOCK_HASH).unwrap();

        let params =
            SolanaTransactionParameters::new(keypair_alice, address_bob, lamports, block_hash);

        let mut transaction = SolanaTransaction::new(&params).unwrap();
        let res = transaction.sign(vec![], 0);
        assert!(res.is_ok());
    }
}
