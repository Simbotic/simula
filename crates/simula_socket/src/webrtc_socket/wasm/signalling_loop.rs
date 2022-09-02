use crate::webrtc_socket::messages::*;
use futures::{SinkExt, StreamExt};
use futures_util::select;
use log::{debug, error};
use ws_stream_wasm::{WsMessage, WsMeta};

pub async fn signalling_loop(
    room_url: String,
    mut requests_receiver: futures_channel::mpsc::UnboundedReceiver<PeerRequest>,
    events_sender: futures_channel::mpsc::UnboundedSender<PeerEvent>,
) {
    let res = WsMeta::connect(&room_url, None).await;
    if let Ok((_ws, wsio)) = res {
        let mut wsio = wsio.fuse();

        loop {
            select! {
                request = requests_receiver.next() => {
                    let request = serde_json::to_string(&request);
                    if let Ok(request) = request {
                        debug!("-> {}", request);
                        let res = wsio.send(WsMessage::Text(request)).await;
                        if res.is_err() {
                            error!("request send error");
                            break;
                        }
                    }
                    else {
                        error!("serializing request");
                        break;
                    }
                }

                message = wsio.next() => {
                    match message {
                        Some(WsMessage::Text(message)) => {
                            debug!("{}", message);
                            let event: Result<PeerEvent,_>  = serde_json::from_str(&message);
                            if let Ok(event) = event {
                                let res = events_sender.unbounded_send(event);
                                if res.is_err() {
                                    error!("Events sender failed to send: {:#?}", res);
                                }
                            }
                            else {
                                error!("Couldn't parse peer event {}", message);
                            }
                        },
                        Some(WsMessage::Binary(_)) => {
                            error!("Received binary data from signal server (expected text). Ignoring.");
                        },
                        None => {
                            error!("Disconnected from signalling server!");
                            break;
                        }
                    }
                }

                complete => break
            }
        }
    } else {
        error!("failed to connect to signalling server");
    }
}
