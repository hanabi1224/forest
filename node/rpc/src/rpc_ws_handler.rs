// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::rpc_util::{call_rpc, call_rpc_str, check_permissions, get_auth_header, get_error_str};
use axum::{
    extract::{
        ws::{Message, WebSocket},
        WebSocketUpgrade,
    },
    response::IntoResponse,
};
use crossbeam::atomic::AtomicCell;
use forest_beacon::Beacon;
use forest_chain::headchange_json::SubscriptionHeadChange;
use forest_rpc_api::{
    chain_api::*,
    data_types::{JsonRpcServerState, StreamingData},
};
use futures::{stream::SplitSink, SinkExt, StreamExt};
use fvm_ipld_blockstore::Blockstore;
use http::{HeaderMap, HeaderValue};
use log::{debug, error, info, warn};
use std::sync::Arc;
use tokio::sync::RwLock;

async fn rpc_ws_task<DB, B>(
    authorization_header: Option<HeaderValue>,
    rpc_call: jsonrpc_v2::RequestObject,
    rpc_server: JsonRpcServerState,
    is_socket_active: Arc<AtomicCell<bool>>,
    ws_sender: Arc<RwLock<SplitSink<WebSocket, Message>>>,
) -> anyhow::Result<()>
where
    DB: Blockstore,
    B: Beacon,
{
    let call_method = rpc_call.method_ref();
    let call_id = rpc_call.id_ref();

    check_permissions::<DB, B>(rpc_server.clone(), call_method, authorization_header)
        .await
        .map_err(|(_, e)| anyhow::Error::msg(e))?;

    match call_method {
        CHAIN_NOTIFY => {
            let request_id = match call_id {
                Some(id) => id.to_owned(),
                None => jsonrpc_v2::Id::Null,
            };

            debug!("Received ChainNotify request with RPC ID: {:?}", request_id);

            let (subscription_response, subscription_id) = call_rpc::<i64>(
                rpc_server.clone(),
                jsonrpc_v2::RequestObject::request()
                    .with_method(CHAIN_HEAD_SUBSCRIPTION)
                    .with_id(request_id.clone())
                    .finish(),
            )
            .await?;

            debug!(
                "Called ChainNotify RPC, got subscription ID {}",
                subscription_id
            );
            {
                ws_sender
                    .write()
                    .await
                    .send(Message::Text(subscription_response))
                    .await?;
            }

            info!(
                "RPC WS ChainNotify for subscription ID: {}",
                subscription_id
            );

            while is_socket_active.load() {
                let (_, event) = call_rpc::<SubscriptionHeadChange>(
                    rpc_server.clone(),
                    jsonrpc_v2::RequestObject::request()
                        .with_method(CHAIN_NOTIFY)
                        .with_id(subscription_id)
                        .finish(),
                )
                .await?;

                debug!("Sending RPC WS ChainNotify event response");

                let event_response = StreamingData {
                    json_rpc: "2.0",
                    method: "xrpc.ch.val",
                    params: event,
                };

                match ws_sender
                    .write()
                    .await
                    .send(Message::Text(serde_json::to_string(&event_response)?))
                    .await
                {
                    Ok(_) => {
                        info!(
                            "New ChainNotify data sent via subscription ID: {}",
                            subscription_id
                        );
                    }
                    Err(msg) => {
                        warn!("WS connection closed. {:?}", msg);
                        is_socket_active.store(false);
                    }
                }
            }
        }
        _ => {
            info!("RPC WS called method: {}", call_method);
            let response = call_rpc_str(rpc_server.clone(), rpc_call).await?;
            ws_sender
                .write()
                .await
                .send(Message::Text(response))
                .await?;
        }
    }

    Ok(())
}

pub async fn rpc_ws_handler<DB, B>(
    headers: HeaderMap,
    axum::Extension(rpc_server): axum::Extension<JsonRpcServerState>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse
where
    DB: Blockstore,
    B: Beacon,
{
    let authorization_header = get_auth_header(headers);
    ws.on_upgrade(move |socket| async {
        rpc_ws_handler_inner::<DB, B>(socket, authorization_header, rpc_server).await
    })
}

async fn rpc_ws_handler_inner<DB, B>(
    socket: WebSocket,
    authorization_header: Option<HeaderValue>,
    rpc_server: JsonRpcServerState,
) where
    DB: Blockstore,
    B: Beacon,
{
    info!("Accepted WS connection!");
    let (sender, mut receiver) = socket.split();
    let ws_sender = Arc::new(RwLock::new(sender));
    let socket_active = Arc::new(AtomicCell::new(true));
    while let Some(Ok(message)) = receiver.next().await {
        debug!("Received new WS RPC message: {:?}", message);
        if let Message::Text(request_text) = message {
            debug!("WS RPC Request: {}", request_text);
            if !request_text.is_empty() {
                info!("RPC Request Received: {:?}", &request_text);
                let authorization_header = authorization_header.clone();
                let task_rpc_server = rpc_server.clone();
                let task_socket_active = socket_active.clone();
                let task_ws_sender = ws_sender.clone();
                match serde_json::from_str(&request_text)
                    as Result<jsonrpc_v2::RequestObject, serde_json::Error>
                {
                    Ok(rpc_call) => {
                        async_std::task::spawn(async move {
                            match rpc_ws_task::<DB, B>(
                                authorization_header,
                                rpc_call,
                                task_rpc_server,
                                task_socket_active,
                                task_ws_sender.clone(),
                            )
                            .await
                            {
                                Ok(_) => {
                                    debug!("WS RPC task success.");
                                }
                                Err(e) => {
                                    let msg = format!("WS RPC task error: {}", e);
                                    error!("{}", msg);
                                    task_ws_sender
                                        .write()
                                        .await
                                        .send(Message::Text(get_error_str(3, msg)))
                                        .await
                                        .unwrap();
                                }
                            }
                        });
                    }
                    Err(e) => {
                        let msg = format!("Error deserializing WS request payload: {}", e);
                        error!("{}", msg);
                        if let Err(e) = task_ws_sender
                            .write()
                            .await
                            .send(Message::Text(get_error_str(1, msg)))
                            .await
                        {
                            warn!("{e}");
                        }
                    }
                }
            }
        }
    }
    socket_active.store(false);
}
