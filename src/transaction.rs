use crate::{SolanaAddress, SolanaFormat, SolanaPublicKey};
use anychain_core::{Transaction, TransactionError, TransactionId};
use solana_sdk::{
    hash::Hash, message::Message, pubkey::Pubkey, signature::Signature,
    system_instruction::transfer as sol_transfer, transaction::Transaction as Tx,
};
use spl_associated_token_account::{
    get_associated_token_address,
    instruction::create_associated_token_account,
};
use spl_token::{id, instruction::transfer_checked as token_transfer};
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SolanaTransactionId {}

impl fmt::Display for SolanaTransactionId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "0xtxid")
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
                        let ix_transfer = token_transfer(
                            &id(),
                            &src,
                            &token,
                            &dest,
                            &from,
                            &[],
                            amount,
                            6,
                        ).unwrap();
                        vec![ix_transfer]
                    }
                    Some(false) => {
                        let ix_create_account = create_associated_token_account(
                            &from,
                            &to,
                            &token,
                            &id(),
                        );
                        let ix_transfer = token_transfer(
                            &id(),
                            &src,
                            &token,
                            &dest,
                            &from,
                            &[],
                            amount,
                            6,
                        ).unwrap();
                        vec![ix_create_account, ix_transfer]
                    }
                    None => return Err(TransactionError::Message("'has_token_account' is not provided".to_string())),
                };
                Message::new_with_blockhash(&ixs,Some(&from), &blockhash)
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

    fn from_bytes(_tx: &[u8]) -> Result<Self, TransactionError> {
        todo!()
    }

    fn to_transaction_id(&self) -> Result<Self::TransactionId, TransactionError> {
        todo!()
    }
}

// mod tests {
//     use super::*;
//     use anychain_core::amount;
//     use solana_rpc_client::rpc_client::RpcClient;
//     use std::net::{TcpListener, TcpStream};
//     use tungstenite::{accept, connect, stream::MaybeTlsStream, Message as WMessage, WebSocket};
//     use url::Url;

//     fn server_send(ws: &mut WebSocket<TcpStream>, s: String) {
//         let msg = WMessage::from(s);
//         ws.write_message(msg).unwrap();
//     }

//     fn server_receive(ws: &mut WebSocket<TcpStream>) -> String {
//         let msg = ws.read_message().unwrap();
//         msg.to_string()
//     }

//     fn client_send(ws: &mut WebSocket<MaybeTlsStream<TcpStream>>, s: String) {
//         let msg = WMessage::from(s);
//         ws.write_message(msg).unwrap();
//     }

//     fn client_receive(ws: &mut WebSocket<MaybeTlsStream<TcpStream>>) -> String {
//         let msg = ws.read_message().unwrap();
//         msg.to_string()
//     }

//     fn server_init() -> WebSocket<TcpStream> {
//         let listener = TcpListener::bind("127.0.0.1:8000").unwrap();
//         let (conn, _) = listener.accept().unwrap();
//         let ws = accept(conn).unwrap();
//         ws
//     }

//     fn client_init() -> WebSocket<MaybeTlsStream<TcpStream>> {
//         connect(Url::parse("ws://127.0.0.1:8000").unwrap()).unwrap().0
//     }

//     #[test]
//     fn f() {
//         let client = RpcClient::new("https://api.mainnet-beta.solana.com");
//         let blockhash = client.get_latest_blockhash().unwrap().to_string();

//         let token = "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB";
//         let from = "8gvxAVripdzJ7nDNt1tPQWtmeHkq2nrgpe1BRYWMWUUo";
//         let to = "84vJfvciURyMSJ6NkKp97L4B4chQzaqyzSjvuoxuLmz6";

//         let token = Some(SolanaAddress(token.to_string()));
//         let from = SolanaAddress(from.to_string());
//         let to = SolanaAddress(to.to_string());
//         let amount = 200u64;

//         let mut tx = SolanaTransaction::new(
//             &SolanaTransactionParameters {
//             token,
//             from,
//             to,
//             amount,
//             blockhash,
//         }).unwrap();

//         let msg = hex::encode(tx.to_bytes().unwrap());
//         let msg = format!("[\"{}\"]", msg);

//         let mut conn = server_init();
//         server_send(&mut conn, msg);

//         let sig = server_receive(&mut conn);
//         let sig = hex::decode(sig).unwrap();

//         let tx = tx.sign(sig, 0).unwrap();
//         let tx = bs58::encode(tx).into_string();

//         println!("tx: {}", tx);
//     }
// }
