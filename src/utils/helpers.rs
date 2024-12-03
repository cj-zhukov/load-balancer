use regex::Error;
use regex::Regex;

pub fn parse_workers_addresses(addresses: &str) -> Vec<String> {
    addresses
        .split(',')
        .map(|s| format!("http://{}", s.trim()))
        .collect::<Vec<_>>()
}

pub fn validate_address(address: &str) -> Result<bool, Error> {
    let re = Regex::new(r"^http://([a-zA-Z0-9.-]+):([0-9]{1,5})$")?;

    if let Some(captures) = re.captures(address) {
        if let Some(port_match) = captures.get(2) {
            let port = &port_match.as_str()[1..];
            if let Ok(_port) = port.parse::<u16>() {
                return Ok(true);
            }
            return Ok(false);
        }
        return Ok(true);
    }
    
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_valid() {
        let addresses = vec![
            "http://example.com:80",
            "http://localhost:8080",
            "http://192.168.1.1:65535",
        ];

        let results = addresses
            .iter()
            .map(|address| validate_address(address).unwrap())
            .collect::<Vec<_>>();

        assert!(results.iter().all(|x| *x == true));
    }


    #[test]
    fn test_validate_invalid() {
        let addresses = vec![
            "http://example.com",           // Invalid: no port
            "http://example.com:999999",    // Invalid: port out of range
            "http://example.com:abcd",      // Invalid: non-numeric port
        ];

        let results = addresses
            .iter()
            .map(|address| validate_address(address).unwrap())
            .collect::<Vec<_>>();

        assert!(results.iter().all(|x| *x == false));
    }
}