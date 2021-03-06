//! A session with a validator node

use std::io::Write;
use std::net::TcpStream;
use std::sync::Arc;

use ed25519::{Keyring, PublicKey};
use error::Error;
use rpc::{Request, Response, SignRequest, SignResponse};

/// A (soon-to-be-encrypted) session with a validator node
pub struct Session {
    /// TCP connection to a validator node
    socket: TcpStream,

    /// Keyring of signature keys
    keyring: Arc<Keyring>,
}

impl Session {
    /// Create a new session with the validator at the given address/port
    pub fn new(addr: &str, port: u16, keyring: Arc<Keyring>) -> Result<Self, Error> {
        debug!("Connecting to {}:{}...", addr, port);
        let socket = TcpStream::connect(format!("{}:{}", addr, port))?;
        Ok(Self { socket, keyring })
    }

    /// Handle an incoming request from the validator
    pub fn handle_request(&mut self) -> Result<bool, Error> {
        let response = match Request::read(&mut self.socket)? {
            Request::Sign(ref req) => self.sign(req)?,
            #[cfg(debug_assertions)]
            Request::PoisonPill => return Ok(false),
        };

        self.socket.write_all(&response.to_vec())?;
        Ok(true)
    }

    /// Perform a digital signature operation
    fn sign(&mut self, request: &SignRequest) -> Result<Response, Error> {
        let pk = PublicKey::from_bytes(&request.public_key)?;
        let signature = self.keyring.sign(&pk, &request.msg)?;

        Ok(Response::Sign(SignResponse {
            sig: signature.as_bytes().to_vec(),
        }))
    }
}
