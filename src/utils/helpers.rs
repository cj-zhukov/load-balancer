pub fn parse_workers_addresses(addresses: &str) -> Vec<String> {
    addresses
        .split(',')
        .map(|s| format!("http://{}", s.trim()))
        .collect::<Vec<_>>()
}