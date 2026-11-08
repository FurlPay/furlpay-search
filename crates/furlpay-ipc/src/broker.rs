//! Per-client SPSC channel allocator.

use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BrokerError {
    #[error("client {0} already has an active channel")]
    ClientExists(u32),
    #[error("client {0} not found")]
    ClientNotFound(u32),
    #[error("maximum client limit ({0}) reached")]
    MaxClientsReached(u32),
}

#[derive(Debug, Clone)]
pub struct BrokerConfig {
    pub max_clients: u32,
    pub slot_size: usize,
    pub slot_count: usize,
    pub shm_base_path: String,
}

impl Default for BrokerConfig {
    fn default() -> Self {
        Self {
            max_clients: 64,
            slot_size: 512,
            slot_count: 16,
            shm_base_path: if cfg!(windows) {
                "Local\\furlpay_client_".to_string()
            } else {
                "/furlpay_client_".to_string()
            },
        }
    }
}

#[derive(Debug)]
pub struct ClientChannel {
    pub client_pid: u32,
    pub shm_name: String,
    pub allocated_bytes: usize,
    pub created_at: u64,
}

pub struct ChannelBroker {
    config: BrokerConfig,
    channels: HashMap<u32, ClientChannel>,
}

impl ChannelBroker {
    pub fn new(config: BrokerConfig) -> Self {
        Self {
            config,
            channels: HashMap::new(),
        }
    }

    pub fn allocate_channel(&mut self, client_pid: u32) -> Result<&ClientChannel, BrokerError> {
        if self.channels.contains_key(&client_pid) {
            return Err(BrokerError::ClientExists(client_pid));
        }
        if self.channels.len() as u32 >= self.config.max_clients {
            return Err(BrokerError::MaxClientsReached(self.config.max_clients));
        }

        let shm_name = format!("{}{}", self.config.shm_base_path, client_pid);
        let allocated_bytes = std::mem::size_of::<crate::ring_buffer::RingBufferHeader>()
            + self.config.slot_size * self.config.slot_count;

        let channel = ClientChannel {
            client_pid,
            shm_name,
            allocated_bytes,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };

        self.channels.insert(client_pid, channel);
        Ok(self.channels.get(&client_pid).unwrap())
    }

    pub fn release_channel(&mut self, client_pid: u32) -> Result<ClientChannel, BrokerError> {
        self.channels
            .remove(&client_pid)
            .ok_or(BrokerError::ClientNotFound(client_pid))
    }

    pub fn get_channel(&self, client_pid: u32) -> Option<&ClientChannel> {
        self.channels.get(&client_pid)
    }

    pub fn active_count(&self) -> usize {
        self.channels.len()
    }
}

impl Default for ChannelBroker {
    fn default() -> Self {
        Self::new(BrokerConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allocate_and_release() {
        let mut broker = ChannelBroker::default();
        let channel = broker.allocate_channel(1234).unwrap();
        assert_eq!(channel.client_pid, 1234);
        assert_eq!(broker.active_count(), 1);
        broker.release_channel(1234).unwrap();
        assert_eq!(broker.active_count(), 0);
    }

    #[test]
    fn test_duplicate_client() {
        let mut broker = ChannelBroker::default();
        broker.allocate_channel(1234).unwrap();
        assert!(broker.allocate_channel(1234).is_err());
    }
}
