use crate::{SolanaAddress, SolanaFormat, SolanaPublicKey};
use anychain_core::{Transaction, TransactionError, TransactionId};
use solana_sdk::{
    hash::Hash,
    message::Message,
    pubkey::Pubkey,
    signature::Signature,
    system_instruction::{transfer as sol_transfer, SystemInstruction},
    transaction::Transaction as Tx,
};
use spl_associated_token_account::{
    get_associated_token_address, instruction::create_associated_token_account,
};
use spl_token::{
    id,
    instruction::{transfer_checked as token_transfer, TokenInstruction},
};
use std::{fmt, str::FromStr};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SolanaTransactionParameters {
    pub token: Option<SolanaAddress>,
    pub has_token_account: Option<bool>,
    pub from: SolanaAddress,
    pub to: SolanaAddress,
    pub amount: u64,
    pub blockhash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SolanaTransaction {
    pub params: SolanaTransactionParameters,
    pub signature: Option<Vec<u8>>,
}

impl FromStr for SolanaTransaction {
    type Err = TransactionError;
    fn from_str(tx: &str) -> Result<Self, Self::Err> {
        let tx = bs58::decode(tx)
            .into_vec()
            .map_err(|e| TransactionError::Message(format!("{}", e)))?;
        SolanaTransaction::from_bytes(&tx)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SolanaTransactionId(pub [u8; 64]);

impl fmt::Display for SolanaTransactionId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", bs58::encode(&self.0.to_vec()).into_string())
    }
}

impl TransactionId for SolanaTransactionId {}

impl Transaction for SolanaTransaction {
    type Address = SolanaAddress;
    type Format = SolanaFormat;
    type PublicKey = SolanaPublicKey;
    type TransactionParameters = SolanaTransactionParameters;
    type TransactionId = SolanaTransactionId;

    fn new(params: &Self::TransactionParameters) -> Result<Self, TransactionError> {
        Ok(SolanaTransaction {
            params: params.clone(),
            signature: None,
        })
    }

    fn sign(&mut self, rs: Vec<u8>, _: u8) -> Result<Vec<u8>, TransactionError> {
        if rs.len() != 64 {
            return Err(TransactionError::Message(format!(
                "Invalid signature length {}",
                rs.len(),
            )));
        }
        self.signature = Some(rs);
        self.to_bytes()
    }

    fn to_bytes(&self) -> Result<Vec<u8>, TransactionError> {
        let from = Pubkey::from_str(&self.params.from.0).unwrap();
        let to = Pubkey::from_str(&self.params.to.0).unwrap();
        let amount = self.params.amount;
        let blockhash = Hash::from_str(&self.params.blockhash).unwrap();

        let msg = match &self.params.token {
            Some(token) => {
                let token = Pubkey::from_str(&token.0).unwrap();
                let src = get_associated_token_address(&from, &token);
                let dest = get_associated_token_address(&to, &token);
                let ixs = match self.params.has_token_account {
                    Some(true) => {
                        let ix_transfer =
                            token_transfer(&id(), &src, &token, &dest, &from, &[], amount, 6)
                                .unwrap();
                        vec![ix_transfer]
                    }
                    Some(false) => {
                        let ix_create_account =
                            create_associated_token_account(&from, &to, &token, &id());
                        let ix_transfer =
                            token_transfer(&id(), &src, &token, &dest, &from, &[], amount, 6)
                                .unwrap();
                        vec![ix_create_account, ix_transfer]
                    }
                    None => {
                        return Err(TransactionError::Message(
                            "'has_token_account' is not provided".to_string(),
                        ))
                    }
                };
                Message::new_with_blockhash(&ixs, Some(&from), &blockhash)
            }
            None => {
                let ix = sol_transfer(&from, &to, amount);
                Message::new_with_blockhash(&[ix], Some(&from), &blockhash)
            }
        };

        match &self.signature {
            Some(rs) => {
                let mut tx = Tx::new_unsigned(msg);
                let mut sig = [0u8; 64];
                sig.copy_from_slice(rs.as_slice());
                tx.signatures = vec![Signature::from(sig)];
                Ok(bincode::serialize(&tx).unwrap())
            }
            None => Ok(msg.serialize()),
        }
    }

    fn from_bytes(tx: &[u8]) -> Result<Self, TransactionError> {
        let tx = bincode::deserialize::<Tx>(tx)
            .map_err(|e| TransactionError::Message(format!("{}", e)))?;

        let sig = if !tx.signatures.is_empty() {
            let rs = tx.signatures[0];
            let mut sig = [0u8; 64];
            sig.copy_from_slice(rs.as_ref());
            Some(sig.to_vec())
        } else {
            None
        };

        let keys = tx.message.account_keys;
        let ixs = tx.message.instructions;
        let blockhash = tx.message.recent_blockhash;

        match ixs.len() {
            1 => {
                let program = keys[ixs[0].program_id_index as usize];
                let account = &ixs[0].accounts;
                let data = &ixs[0].data;
                match format!("{}", program).as_str() {
                    "11111111111111111111111111111111" => {
                        let from = keys[account[0] as usize];
                        let to = keys[account[1] as usize];

                        let ix = bincode::deserialize::<SystemInstruction>(data)
                            .map_err(|e| TransactionError::Message(format!("{}", e)))?;

                        match ix {
                            SystemInstruction::Transfer { lamports } => {
                                let params = SolanaTransactionParameters {
                                    token: None,
                                    has_token_account: None,
                                    from: SolanaAddress(from.to_string()),
                                    to: SolanaAddress(to.to_string()),
                                    amount: lamports,
                                    blockhash: blockhash.to_string(),
                                };
                                let mut tx = SolanaTransaction::new(&params)?;
                                tx.signature = sig;
                                Ok(tx)
                            }
                            _ => Err(TransactionError::Message(format!(
                                "Unsupported system instruction: {:?}",
                                ix
                            ))),
                        }
                    }
                    "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA" => {
                        let token = keys[account[1] as usize];
                        let dest = keys[account[2] as usize];
                        let from = keys[account[3] as usize];

                        let ix = TokenInstruction::unpack(data)
                            .map_err(|e| TransactionError::Message(format!("{}", e)))?;

                        match ix {
                            TokenInstruction::TransferChecked { amount, .. } => {
                                let params = SolanaTransactionParameters {
                                    token: Some(SolanaAddress(token.to_string())),
                                    has_token_account: Some(true),
                                    from: SolanaAddress(from.to_string()),
                                    to: SolanaAddress(dest.to_string()),
                                    amount,
                                    blockhash: blockhash.to_string(),
                                };
                                let mut tx = SolanaTransaction::new(&params)?;
                                tx.signature = sig;
                                Ok(tx)
                            }
                            _ => Err(TransactionError::Message(format!(
                                "Unsupported token instruction: {:?}",
                                ix
                            ))),
                        }
                    }
                    _ => Err(TransactionError::Message(format!(
                        "Unsupported program {}",
                        program
                    ))),
                }
            }
            2 => {
                let program1 = keys[ixs[0].program_id_index as usize];
                let program2 = keys[ixs[1].program_id_index as usize];

                if format!("{}", program1).as_str()
                    != "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"
                {
                    return Err(TransactionError::Message(format!(
                        "Unsupported first program {}",
                        program1
                    )));
                }

                if format!("{}", program2).as_str() != "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
                {
                    return Err(TransactionError::Message(format!(
                        "Unsupported second program {}",
                        program2
                    )));
                }

                let account = &ixs[0].accounts;
                let data = &ixs[1].data;

                let funding_address = keys[account[0] as usize];
                let funded_address = keys[account[2] as usize];
                let token_address = keys[account[3] as usize];

                let ix = TokenInstruction::unpack(data)
                    .map_err(|e| TransactionError::Message(format!("{}", e)))?;

                match ix {
                    TokenInstruction::TransferChecked { amount, .. } => {
                        let params = SolanaTransactionParameters {
                            token: Some(SolanaAddress(token_address.to_string())),
                            has_token_account: Some(false),
                            from: SolanaAddress(funding_address.to_string()),
                            to: SolanaAddress(funded_address.to_string()),
                            amount,
                            blockhash: blockhash.to_string(),
                        };
                        let mut tx = SolanaTransaction::new(&params)?;
                        tx.signature = sig;
                        Ok(tx)
                    }
                    _ => Err(TransactionError::Message(format!(
                        "Unsupported token instruction: {:?}",
                        ix
                    ))),
                }
            }
            _ => Err(TransactionError::Message(format!(
                "Unsupported instruction amount: {}",
                ixs.len()
            ))),
        }
    }

    fn to_transaction_id(&self) -> Result<Self::TransactionId, TransactionError> {
        match &self.signature {
            Some(sig) => {
                let mut txid = [0u8; 64];
                txid.copy_from_slice(sig);
                Ok(SolanaTransactionId(txid))
            }
            None => Err(TransactionError::Message(
                "Transaction is not signed".to_string(),
            )),
        }
    }
}

#[test]
fn test() {
    let tx = "BU8oN58NjvzGdbuQ8zGKF9cJ7N25iWRRgnLodf42gEVDnzcQ3g5y7eygBviCRQHH4sC335gt575JA2NfjpX3P7m1vZ5WYWxHem7wW3Pc4S6YYi4ftivYiGqTMr6eKtUVCbBZabwyMuZ7iGjUtTB6L7LnfQj6wGduNUqwpGPy2xD8aFps6zRfgwNAXe9tpoa3tQvTnyU8WgkpiZjkBFdfXFw8abhsUZLZsxaYra2CHmqrXwG6VFUfhTdYANPTXcBcZ2a75RmqC19d5rYJPexmpGJV529A4WXgE4Pm5Gk5AUB7LcNmAxfkKxJk3ikGohb9n3B7vJ3T9zJZg4i6xEGapobavsLwMuYkCjnRBQ69rouMCJEtz33XNuwx1ZN84cGimZV1KSbwQgcPDFzgdZR2ZisViDWAJUXkadfCfADNEME1jxmHDy7oX9gTYJvkeZAnoFjxVhKrVZft8FaADcRgNcdZJPdt9rMMSpCJXBFgBVsGaqo6iteJqg79qQrEoScRviUh6scB7iwCh";
    let tx = SolanaTransaction::from_str(tx).unwrap();
    let txid = tx.to_transaction_id().unwrap();
    println!("{}", txid);
}
