struct TlsCert {
    last_renewed: u64,
    renew_at: u64,
    priv_key: Vec<u8>,
    pub_cert_chain: Vec<u8>,
}

pub(crate) struct ServerSettings {}
