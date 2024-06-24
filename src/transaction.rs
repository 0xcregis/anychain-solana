use crate::{SolanaAddress, SolanaFormat, SolanaPublicKey};
use anychain_core::{Transaction, TransactionError, TransactionId};
use solana_sdk::{
    hash::Hash, message::Message, pubkey::Pubkey, signature::Signature,
    system_instruction::transfer, transaction::Transaction as Tx,
};
use std::{fmt, str::FromStr};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SolanaTransactionParameters {
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

    fn new(params: &Self::TransactionParameters) -> Result<Self, anychain_core::TransactionError> {
        Ok(SolanaTransaction {
            params: params.clone(),
            signature: None,
        })
    }

    fn sign(&mut self, rs: Vec<u8>, _: u8) -> Result<Vec<u8>, anychain_core::TransactionError> {
        if rs.len() != 64 {
            return Err(TransactionError::Message(format!(
                "Invalid signature length {}",
                rs.len(),
            )));
        }
        self.signature = Some(rs);
        self.to_bytes()
    }

    fn to_bytes(&self) -> Result<Vec<u8>, anychain_core::TransactionError> {
        let from = Pubkey::from_str(&self.params.from.0).unwrap();
        let to = Pubkey::from_str(&self.params.to.0).unwrap();
        let amount = self.params.amount;
        let blockhash = Hash::from_str(&self.params.blockhash).unwrap();
        let ins = transfer(&from, &to, amount);
        let msg = Message::new_with_blockhash(&[ins], Some(&from), &blockhash);
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

    fn from_bytes(_tx: &[u8]) -> Result<Self, anychain_core::TransactionError> {
        todo!()
    }

    fn to_transaction_id(&self) -> Result<Self::TransactionId, anychain_core::TransactionError> {
        todo!()
    }
}

// mod tests {
//     use super::*;
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
//         let client = RpcClient::new("https://api.devnet.solana.com");
//         let blockhash = client.get_latest_blockhash().unwrap();
//         let from = [64, 7, 30, 154, 231, 4, 201, 240, 49, 135, 104, 181, 174, 183, 202, 2, 185, 54, 230, 84, 43, 113, 54, 194, 158, 123, 200, 43, 30, 208, 64, 142];
//         let from = Pubkey::new_from_array(from);
//         let to = Pubkey::from_str("AN6yLuRMsyCsj178nhihPdX3DaKUYsaRdD5L3DfNWPDZ").unwrap();
//         let amount = 1000000000u64;

//         println!("{}\n{}", from, to);

//         let ins = transfer(&from, &to, amount);
//         let msg = Message::new_with_blockhash(&[ins], Some(&from), &blockhash);

//         let mut tx = Tx::new_unsigned(msg);
//         let msg = hex::encode(tx.message_data());
//         let msg = format!("[\"{}\"]", msg);

//         let mut conn = server_init();
//         server_send(&mut conn, msg);

//         let rs = server_receive(&mut conn);

//         let mut sig = [0u8; 64];
//         let rs = hex::decode(&rs).unwrap();
//         sig.copy_from_slice(&rs);

//         let sig = Signature::from(sig);

//         tx.signatures = vec![sig];

//         let tx = bincode::serialize(&tx).unwrap();
//         let tx = bs58::encode(&tx).into_string();

//         println!("tx = {}", tx);
//     }
// }
