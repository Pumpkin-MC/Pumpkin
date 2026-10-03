use std::{
    net::{IpAddr, SocketAddr},
    sync::{Arc, Weak},
};

use pumpkin_auth::{jwt::Jwks, p384::ecdsa::SigningKey};
use tokio::sync::{OnceCell, mpsc};

use crate::server::Server;

use super::{ice_router::IceRouter, session::IncomingSession};

#[derive(Clone)]
pub(super) struct NetherNetState {
    pub(super) server: Weak<Server>,
    pub(super) incoming: mpsc::Sender<IncomingSession>,
    pub(super) identity_key: Arc<SigningKey>,
    pub(super) require_client_identity: bool,
    pub(super) oidc_verifier: Option<Arc<OnceCell<(String, Jwks)>>>,
    pub(super) stun_servers: Arc<[String]>,
    pub(super) ice_local_addr: SocketAddr,
    pub(super) external_ip: Option<IpAddr>,
    pub(super) ice_router: Arc<IceRouter>,
}
