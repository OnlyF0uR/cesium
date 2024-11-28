use std::sync::Arc;

use cesium_nebula::transaction::{Transaction, TransactionError};
use cesium_nucleus::graph::mempool::Graph;
use hex::FromHexError;
use jsonrpsee::{
    core::{async_trait, SubscriptionResult},
    proc_macros::rpc,
    types::ErrorObject,
    ConnectionId, PendingSubscriptionSink,
};
use jsonrpsee::{Extensions, SubscriptionMessage};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug)]
pub enum RpcError {
    IoError(std::io::Error),
    HexError(hex::FromHexError),
    RpcError(String),
    TxError(TransactionError),
    GraphError(cesium_nucleus::graph::errors::GraphError),
}

impl From<RpcError> for ErrorObject<'static> {
    fn from(err: RpcError) -> Self {
        match err {
            RpcError::IoError(e) => ErrorObject::owned(1, "IO Error", Some(e.to_string())),
            RpcError::HexError(e) => ErrorObject::owned(2, "Hex Error", Some(e.to_string())),
            RpcError::RpcError(e) => ErrorObject::owned(2, "RPC Error", Some(e)),
            RpcError::TxError(e) => ErrorObject::owned(2, "Transaction Error", Some(e.to_string())),
            RpcError::GraphError(e) => ErrorObject::owned(2, "Graph Error", Some(e.to_string())),
        }
    }
}

impl From<std::io::Error> for RpcError {
    fn from(e: std::io::Error) -> Self {
        RpcError::IoError(e)
    }
}

impl From<FromHexError> for RpcError {
    fn from(e: FromHexError) -> Self {
        RpcError::HexError(e)
    }
}

impl From<TransactionError> for RpcError {
    fn from(e: TransactionError) -> Self {
        RpcError::TxError(e)
    }
}

impl From<cesium_nucleus::graph::errors::GraphError> for RpcError {
    fn from(e: cesium_nucleus::graph::errors::GraphError) -> Self {
        RpcError::GraphError(e)
    }
}

#[rpc(server)]
pub trait Rpc {
    #[method(name = "getVersion")]
    async fn get_version(&self) -> Result<String, RpcError>;

    // getCheckpoint is a method that returns the checkpoint at the given index.
    // If no index is provided, it returns the latest checkpoint.
    #[method(name = "getCheckpoint")]
    async fn get_checkpoint(&self, index: Option<u64>) -> Result<String, RpcError>;

    // getTransaction is a method that returns the transaction data given a transaction hash.
    #[method(name = "getTransaction")]
    async fn get_transaction(&self, hash: String) -> Result<String, RpcError>;

    // sendTransaction is a method that sends a transaction to the network.
    #[method(name = "sendTransaction")]
    async fn send_transaction(&self, tx: String) -> Result<String, RpcError>;

    // getAccountInfo is a method that returns the account information for a given account.
    // This can be called on base accounts, as well as on data accounts
    #[method(name = "getAccountInfo")]
    async fn get_account_info(&self, account: String) -> Result<String, RpcError>;

    // checkpointsSub is a subscription method that broadcasts the latest checkpoint information.
    #[subscription(name = "subscribeCheckpoints", item = usize, with_extensions)]
    async fn checkpoints_sub(&self) -> SubscriptionResult;

    // transactionsSub is a subscription method that broadcasts the latest transaction information.
    #[subscription(name = "subscribeTransactions", item = usize, with_extensions)]
    async fn transactions_sub(&self) -> SubscriptionResult;

    // accountSub is a subscription method that broadcasts the latest account information.
    #[subscription(name = "subscribeAccount", item = usize, with_extensions)]
    async fn account_sub(&self) -> SubscriptionResult;
}

pub struct RpcServerImpl {
    dag: Arc<Graph<'static>>, // If possible, make Graph 'static
}

impl RpcServerImpl {
    pub fn new(dag: Arc<Graph<'static>>) -> Self {
        Self { dag }
    }
}

#[async_trait]
impl RpcServer for RpcServerImpl {
    async fn get_version(&self) -> Result<String, RpcError> {
        Ok(VERSION.to_string())
    }

    async fn get_checkpoint(&self, _index: Option<u64>) -> Result<String, RpcError> {
        Ok("todo".to_string())
    }

    async fn get_transaction(&self, _hash: String) -> Result<String, RpcError> {
        Ok("todo".to_string())
    }

    async fn send_transaction(&self, tx: String) -> Result<String, RpcError> {
        let bytes = hex::decode(tx)?;
        let tx = Transaction::from_bytes(&bytes)?;
        if !tx.is_signed() {
            return Err(TransactionError::NotSigned.into());
        }
        if !tx.verify()? {
            return Err(TransactionError::InvalidSignature.into());
        }

        // TODO: May still need to do some things here?
        self.dag.add_item(&tx).await?;

        Ok("todo".to_string())
    }

    async fn get_account_info(&self, _account: String) -> Result<String, RpcError> {
        Ok("todo".to_string())
    }

    async fn checkpoints_sub(
        &self,
        pending: PendingSubscriptionSink,
        ext: &Extensions,
    ) -> SubscriptionResult {
        let sink = pending.accept().await?;
        let conn_id = ext
            .get::<ConnectionId>()
            .cloned()
            .ok_or_else(|| ErrorObject::owned(0, "No connection details found", None::<()>))?;

        // TODO: Implement saving the connection for future use

        sink.send(SubscriptionMessage::from_json(&conn_id).unwrap())
            .await?;
        Ok(())
    }

    async fn transactions_sub(
        &self,
        pending: PendingSubscriptionSink,
        ext: &Extensions,
    ) -> SubscriptionResult {
        let sink = pending.accept().await?;
        let conn_id = ext
            .get::<ConnectionId>()
            .cloned()
            .ok_or_else(|| ErrorObject::owned(0, "No connection details found", None::<()>))?;

        // TODO: Implement saving the connection for future use

        sink.send(SubscriptionMessage::from_json(&conn_id).unwrap())
            .await?;

        Ok(())
    }

    async fn account_sub(
        &self,
        pending: PendingSubscriptionSink,
        ext: &Extensions,
    ) -> SubscriptionResult {
        let sink = pending.accept().await?;
        let conn_id = ext
            .get::<ConnectionId>()
            .cloned()
            .ok_or_else(|| ErrorObject::owned(0, "No connection details found", None::<()>))?;

        // TODO: Implement saving the connection for future use

        sink.send(SubscriptionMessage::from_json(&conn_id).unwrap())
            .await?;

        Ok(())
    }
}

pub async fn start_rpc(dag: &Arc<Graph<'static>>) -> Result<String, RpcError> {
    let rpc_middleware = jsonrpsee::server::middleware::rpc::RpcServiceBuilder::new();
    let server = jsonrpsee::server::Server::builder()
        .set_rpc_middleware(rpc_middleware)
        .build("127.0.0.1:0")
        .await?;

    let addr = server.local_addr()?;
    let rpc_server = RpcServerImpl::new(Arc::clone(dag));
    let handle = server.start(rpc_server.into_rpc());

    tokio::spawn(handle.stopped());

    let url = format!("ws://{}", addr);
    Ok(url)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use cesium_crypto::mldsa::keypair::SignerPair;
    use cesium_nucleus::graph::mempool::Graph;
    use jsonrpsee::{
        core::{client::ClientT, ClientError},
        rpc_params,
        ws_client::WsClientBuilder,
    };

    #[tokio::test]
    async fn test_get_version() {
        // Create the account and wrap it in Arc
        let acc = Box::leak(Box::new(SignerPair::create()));
        let dag = Arc::new(Graph::default(acc));

        let url = super::start_rpc(&dag).await.unwrap();

        let client = WsClientBuilder::default().build(&url).await.unwrap();
        let result: Result<String, ClientError> = client.request("getVersion", rpc_params!()).await;

        let env_version = env!("CARGO_PKG_VERSION");
        assert_eq!(result.unwrap(), env_version.to_string());
    }
}
