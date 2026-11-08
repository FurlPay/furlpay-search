//! HTTP/3 QUIC connection pool (Milestone 2 stub).
pub struct QuicPool;
impl QuicPool {
    pub async fn get(_url: &str) -> Result<Vec<u8>, std::io::Error> {
        Ok(Vec::new())
    }
}
