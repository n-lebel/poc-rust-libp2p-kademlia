use std::collections::HashMap;
use std::env::args;
use std::error::Error;
use std::time::Duration;

use tokio;
use log::{info, error, warn};
use env_logger::{Env, Builder};
use std::num::NonZeroUsize;

use libp2p::{
    Multiaddr,
    identity, 
    PeerId,
    StreamProtocol, 
    SwarmBuilder,
};

use libp2p::futures::StreamExt;
use libp2p::swarm::SwarmEvent;
use libp2p::noise::Config as NoiceConfig;

use libp2p::identify::{
    Config as IdentifyConfig, 
    Behaviour as IdentifyBehavior, 
    Event as IdentifyEvent
};

use libp2p::kad::{
    RoutingUpdate,
    Config as KadConfig, 
    Behaviour as KadBehavior, 
    Event as KadEvent,
    store::MemoryStore as KadInMemory, 
};

use libp2p::request_response::{
    Config as RequestResponseConfig, 
    ProtocolSupport as RequestResponseProtocolSupport,
    Event as RequestResponseEvent,
    Message as RequestResponseMessage
};

use libp2p::request_response::cbor::Behaviour as RequestResponseBehavior;

mod behavior;
use behavior::{
    Behavior as AgentBehavior, 
    Event as AgentEvent
};

mod message;
use message::{GreeRequest, GreetResponse};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    Builder::from_env(Env::default().default_filter_or("debug")).init();

    let local_key = identity::Keypair::generate_ed25519();

    let mut swarm = SwarmBuilder::with_existing_identity(local_key.clone())
        .with_tokio()
        .with_quic()
        .with_behaviour(|key| {

            let local_peer_id = PeerId::from(key.clone().public());
            info!("LocalPeerID: {local_peer_id}");

            let mut kad_config = KadConfig::default();
            kad_config.set_protocol_names(vec![StreamProtocol::new("/agent/connection/1.0.0")]);

            let kad_memory = KadInMemory::new(local_peer_id);
            let kad = KadBehavior::with_config(local_peer_id, kad_memory, kad_config);

            let identity_config = IdentifyConfig::new(
                "/agent/connection/1.0.0".to_string(), 
                key.clone().public()
            )
            .with_push_listen_addr_updates(true)
            .with_interval(Duration::from_secs(30));

            let rr_config = RequestResponseConfig::default();
            let rr_protocol = StreamProtocol::new("/agent/message/1.0.0");
            let rr_behavior = RequestResponseBehavior::<GreeRequest, GreetResponse>::new([(rr_protocol, RequestResponseProtocolSupport::Full)], rr_config);

            let identify = IdentifyBehavior::new(identity_config);
            AgentBehavior::new(kad, identify, rr_behavior)

        })?
        .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(30)).with_notify_handler_buffer_size(NonZeroUsize::new(10000).unwrap()))
        .build();

    swarm.behaviour_mut().set_server_mode();
    
    if let Some(addr) = args().nth(1) {
        swarm.listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse()?)?;
        
        let remote: Multiaddr = addr.parse()?;
        swarm.dial(remote)?;
        info!("Dialed to: {addr}");
    } else {
        info!("Act as bootstrap node");
        swarm.listen_on("/ip4/0.0.0.0/udp/8000/quic-v1".parse()?)?;
    }

    let mut peers: HashMap<PeerId, Vec<Multiaddr>> = HashMap::new();
    let mut received_messages = 0;

    loop {
        match swarm.select_next_some().await {
            SwarmEvent::NewListenAddr { listener_id, address } => info!("NewListenAddr: {listener_id:?} | {address:?}"),
            SwarmEvent::ConnectionEstablished { 
                peer_id, 
                connection_id, 
                endpoint, 
                num_established, 
                concurrent_dial_errors, 
                established_in } => info!("ConnectionEstablished: {peer_id} | {connection_id} | {endpoint:?} | {num_established} | {concurrent_dial_errors:?} | {established_in:?}"),
            SwarmEvent::Dialing { peer_id, connection_id } => info!("Dialing: {peer_id:?} | {connection_id}"),
            SwarmEvent::Behaviour(AgentEvent::Identify(event)) => match event {
                IdentifyEvent::Sent { peer_id } => info!("IdentifyEvent:Sent: {peer_id}"),
                IdentifyEvent::Pushed { peer_id, info } => info!("IdentifyEvent:Pushed: {peer_id} | {info:?}"),
                IdentifyEvent::Received { peer_id, info } => {
                    info!("IdentifyEvent:Received: {peer_id} | {info:?}");
                    // As soon as we connect to a peer, send a large number of messages
                    loop_send_message(&mut swarm, peer_id, GreeRequest{ message: "Hello gaess".to_string() }).await;
                },
                _ => {}
            },
            SwarmEvent::Behaviour(AgentEvent::RequestResponse(event)) => match event {
                RequestResponseEvent::Message { peer, message } => {
                    received_messages += 1;
                    info!("Received messages: {received_messages}");
                },
                RequestResponseEvent::InboundFailure { peer, request_id, error } => {
                    warn!("RequestResponseEvent::InboundFailure -> PeerID: {peer} | RequestID: {request_id} | Error: {error}")
                },
                RequestResponseEvent::OutboundFailure { peer, request_id, error } => {
                    error!("RequestResponseEvent::OutboundFailure -> PeerID: {peer} | RequestID: {request_id} | Error: {error}")
                }
                _ => {}
            },
            _ => {}
        }
    }
}

async fn loop_send_message(swarm: &mut libp2p::Swarm<AgentBehavior>, peer_id: PeerId, message: GreeRequest) {
    (0..500).for_each(|_| {
        let request_id = swarm.behaviour_mut().send_message(&peer_id, message.clone());
        info!("RequestID: {request_id}")
    });
}