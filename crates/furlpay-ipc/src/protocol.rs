//! Wire protocol for shared-memory IPC messages.

/// Query type discriminator for search requests.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryType {
    Exact = 0,
    Regex = 1,
    Symbol = 2,
    Hybrid = 3,
    Web = 4,
}

impl From<u8> for QueryType {
    fn from(v: u8) -> Self {
        match v {
            0 => QueryType::Exact,
            1 => QueryType::Regex,
            2 => QueryType::Symbol,
            3 => QueryType::Hybrid,
            4 => QueryType::Web,
            _ => QueryType::Hybrid,
        }
    }
}

/// Search request message sent from client to daemon via shared memory.
#[repr(C, align(64))]
#[derive(Debug, Clone, Copy)]
pub struct SearchRequestMessage {
    pub request_id: u64,
    pub tenant_token_hash: [u8; 32],
    pub query_type: u8,
    pub timeout_micros: u32,
    pub max_results: u16,
    pub query_len: u16,
    pub query_bytes: [u8; 256],
    pub path_filter: [u8; 128],
}

impl SearchRequestMessage {
    pub fn new() -> Self {
        Self {
            request_id: 0,
            tenant_token_hash: [0u8; 32],
            query_type: QueryType::Hybrid as u8,
            timeout_micros: 500,
            max_results: 10,
            query_len: 0,
            query_bytes: [0u8; 256],
            path_filter: [0u8; 128],
        }
    }

    pub fn set_query(&mut self, query: &str) {
        let bytes = query.as_bytes();
        let len = bytes.len().min(256);
        self.query_bytes[..len].copy_from_slice(&bytes[..len]);
        self.query_len = len as u16;
    }

    pub fn query(&self) -> &str {
        let len = self.query_len as usize;
        std::str::from_utf8(&self.query_bytes[..len]).unwrap_or("")
    }
}

impl Default for SearchRequestMessage {
    fn default() -> Self {
        Self::new()
    }
}

/// A single search hit in the response.
#[repr(C, align(64))]
#[derive(Debug, Clone, Copy)]
pub struct SearchHit {
    pub doc_id: u32,
    pub byte_offset: u32,
    pub match_length: u16,
    pub line_number: u32,
    pub rrf_score: f32,
    pub ast_node_type: u16,
}

impl Default for SearchHit {
    fn default() -> Self {
        Self {
            doc_id: 0,
            byte_offset: 0,
            match_length: 0,
            line_number: 0,
            rrf_score: 0.0,
            ast_node_type: 0,
        }
    }
}

/// Response status codes.
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusCode {
    Success = 200,
    Unauthorized = 401,
    Timeout = 408,
    InternalError = 500,
}

/// Search response message sent from daemon to client via shared memory.
#[repr(C, align(64))]
#[derive(Debug, Clone, Copy)]
pub struct SearchResponseMessage {
    pub request_id: u64,
    pub elapsed_micros: u32,
    pub total_hits: u32,
    pub returned_hits: u16,
    pub generation_id: u64,
    pub status_code: u16,
    pub hits: [SearchHit; 32],
}

impl SearchResponseMessage {
    pub fn new(request_id: u64) -> Self {
        Self {
            request_id,
            elapsed_micros: 0,
            total_hits: 0,
            returned_hits: 0,
            generation_id: 0,
            status_code: StatusCode::Success as u16,
            hits: [SearchHit::default(); 32],
        }
    }

    pub fn error(request_id: u64, status: StatusCode) -> Self {
        let mut msg = Self::new(request_id);
        msg.status_code = status as u16;
        msg
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_alignment() {
        assert_eq!(std::mem::align_of::<SearchRequestMessage>(), 64);
    }

    #[test]
    fn test_response_alignment() {
        assert_eq!(std::mem::align_of::<SearchResponseMessage>(), 64);
    }

    #[test]
    fn test_set_query() {
        let mut req = SearchRequestMessage::new();
        req.set_query("handlePayment");
        assert_eq!(req.query(), "handlePayment");
        assert_eq!(req.query_len, 13);
    }
}
