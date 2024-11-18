use std::net::SocketAddr;

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
    RpcError(String),
}

impl From<RpcError> for ErrorObject<'static> {
    fn from(err: RpcError) -> Self {
        match err {
            RpcError::IoError(e) => ErrorObject::owned(1, "IO Error", Some(e.to_string())),
            RpcError::RpcError(e) => ErrorObject::owned(2, "RPC Error", Some(e)),
        }
    }
}

impl From<std::io::Error> for RpcError {
    fn from(e: std::io::Error) -> Self {
        RpcError::IoError(e)
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

pub struct RpcServerImpl;

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

async fn run_server() -> Result<SocketAddr, RpcError> {
    let rpc_middleware = jsonrpsee::server::middleware::rpc::RpcServiceBuilder::new();
    let server = jsonrpsee::server::Server::builder()
        .set_rpc_middleware(rpc_middleware)
        .build("127.0.0.1:0")
        .await?;

    let addr = server.local_addr()?;
    let handle = server.start(RpcServerImpl.into_rpc());

    tokio::spawn(handle.stopped());

    Ok(addr)
}

pub async fn start_rpc() -> Result<String, RpcError> {
    let server_addr = run_server().await?;
    let url = format!("ws://{}", server_addr);
    Ok(url)
}

#[cfg(test)]
mod tests {
    use jsonrpsee::{
        core::{client::ClientT, ClientError},
        rpc_params,
        ws_client::WsClientBuilder,
    };

    #[tokio::test]
    async fn test_get_version() {
        let url = super::start_rpc().await.unwrap();

        let client = WsClientBuilder::default().build(&url).await.unwrap();
        let result: Result<String, ClientError> = client.request("getVersion", rpc_params!()).await;

        let env_version = env!("CARGO_PKG_VERSION");
        assert_eq!(result.unwrap(), env_version.to_string());
    }
}
