pub mod rpc {
    tonic::include_proto!("concordium");
}

use crate::utils::env;
use http::uri::Uri;
use rpc::p2p_client::P2pClient;
use rpc::{Empty, GetAddressInfoRequest, JsonResponse};
use serde_json::Value;
use tonic::{metadata::MetadataValue, transport::Channel, Request, Response};

type Error = Box<dyn std::error::Error + Send + Sync>;

fn parse_response(response: Response<JsonResponse>) -> Result<Value, serde_json::Error> {
    let json = response.into_inner().value;
    Ok(serde_json::from_str(&json)?)
}

pub async fn get_account_balance(address: &str) -> Result<Option<String>, Error> {
    let uri: Uri = env("CONCORDIUM_GRPC_URL").parse().unwrap();
    let channel = Channel::builder(uri).connect().await?;

    let token = env("CONCORDIUM_GRPC_TOKEN");
    let token = MetadataValue::from_str(&token)?;

    let mut client = P2pClient::with_interceptor(channel, move |mut req: Request<()>| {
        req.metadata_mut().insert("authentication", token.clone());
        Ok(req)
    });

    // Get consensus status
    let resp = client.get_consensus_status(Empty {}).await?;
    let consensus = parse_response(resp)?;

    // Get address info
    let request = tonic::Request::new(GetAddressInfoRequest {
        block_hash: consensus["bestBlock"].as_str().unwrap().to_string(),
        address: address.into(),
    });

    let resp = client.get_account_info(request).await?;
    let json = parse_response(resp)?;

    if json.is_object() {
        Ok(json["accountAmount"].as_str().map(ToOwned::to_owned))
    } else {
        Ok(None)
    }
}
