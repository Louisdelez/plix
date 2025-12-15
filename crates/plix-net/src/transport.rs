//! UDP socket wrapper for Plix networking

use std::io;
use std::net::SocketAddr;

use tokio::net::UdpSocket;
use tracing::{debug, trace};

/// UDP transport for sending and receiving packets
pub struct UdpTransport {
    socket: UdpSocket,
    recv_buf: Vec<u8>,
}

impl UdpTransport {
    /// Create a new UDP transport bound to the given address
    pub async fn bind(addr: SocketAddr) -> io::Result<Self> {
        let socket = UdpSocket::bind(addr).await?;
        debug!(addr = %socket.local_addr()?, "UDP socket bound");

        Ok(Self {
            socket,
            recv_buf: vec![0u8; 1500], // MTU size buffer
        })
    }

    /// Get the local address this transport is bound to
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.socket.local_addr()
    }

    /// Send a packet to the given address
    pub async fn send_to(&self, data: &[u8], addr: SocketAddr) -> io::Result<usize> {
        trace!(to = %addr, len = data.len(), "Sending packet");
        self.socket.send_to(data, addr).await
    }

    /// Receive a packet, returning the data slice and sender address
    pub async fn recv_from(&mut self) -> io::Result<(&[u8], SocketAddr)> {
        let (len, addr) = self.socket.recv_from(&mut self.recv_buf).await?;
        trace!(from = %addr, len = len, "Received packet");
        Ok((&self.recv_buf[..len], addr))
    }

    /// Set the socket to non-blocking mode
    pub fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        self.socket.set_broadcast(nonblocking)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_transport_bind() {
        let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
        let transport = UdpTransport::bind(addr).await.unwrap();
        assert!(transport.local_addr().unwrap().port() != 0);
    }

    #[tokio::test]
    async fn test_transport_send_recv() {
        let addr1: SocketAddr = "127.0.0.1:0".parse().unwrap();
        let addr2: SocketAddr = "127.0.0.1:0".parse().unwrap();

        let transport1 = UdpTransport::bind(addr1).await.unwrap();
        let mut transport2 = UdpTransport::bind(addr2).await.unwrap();

        let addr1_actual = transport1.local_addr().unwrap();
        let addr2_actual = transport2.local_addr().unwrap();

        let data = b"hello";
        transport1.send_to(data, addr2_actual).await.unwrap();

        let (recv_data, from) = transport2.recv_from().await.unwrap();
        assert_eq!(recv_data, data);
        assert_eq!(from, addr1_actual);
    }
}
