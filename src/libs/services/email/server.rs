use super::{Email, EmailError};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

const MAX_LINE_BYTES: usize = 64 * 1024;
const MAX_MESSAGE_BYTES: usize = 10 * 1024 * 1024;

pub type Mailbox = Arc<Mutex<Vec<Email>>>;
pub struct SmtpCatcherServer {
    listener: TcpListener,
    hostname: String,
    mailbox: Mailbox,
}

impl SmtpCatcherServer {
    pub async fn bind(addr: &str) -> std::io::Result<Self> {
        let listener = TcpListener::bind(addr).await?;
        Ok(Self {
            listener,
            hostname: "cctf-catcher".to_string(),
            mailbox: Arc::new(Mutex::new(Vec::new())),
        })
    }
    pub fn with_hostname(mut self, hostname: impl Into<String>) -> Self {
