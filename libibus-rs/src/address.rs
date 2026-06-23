use std::fs;
use std::path::PathBuf;

use crate::error::{Error, Result};

pub(crate) struct Address {
    pub host: String,
    pub port: u32,
    pub socket_path: String,
}

pub(crate) fn read_ibus_address(display: Option<&str>) -> Result<Address> {
    let display = display.unwrap_or(":0");
    let display_num = display
        .trim_start_matches(':')
        .split('.')
        .next()
        .unwrap_or("0");

    let addr_path = get_address_file_path(display_num)?;
    let contents = fs::read_to_string(&addr_path)
        .map_err(|e| Error::Address(format!("Cannot read address file {:?}: {}", addr_path, e)))?;

    parse_address(&contents)
}

fn get_address_file_path(display_num: &str) -> Result<PathBuf> {
    let home = std::env::var("HOME")
        .map_err(|_| Error::Address("HOME environment variable not set".into()))?;

    let config_dir =
        std::env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| format!("{}/.config", home));

    let addr_dir = format!("{}/ibus/bus", config_dir);

    if let Ok(entries) = fs::read_dir(&addr_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            // Reject symlinks and non-regular files to prevent symlink-based attacks
            if let Ok(meta) = path.symlink_metadata() {
                if !meta.is_file() {
                    continue;
                }
            }
            if let Some(name) = path.file_name() {
                let name = name.to_string_lossy();
                let display_suffix = format!("-unix-{}", display_num);
                let display_suffix_short = format!("-{}", display_num);
                if name.starts_with(&format!("{}-", display_num))
                    || name == display_num
                    || name.ends_with(&display_suffix)
                    || name.ends_with(&display_suffix_short)
                {
                    return Ok(path);
                }
            }
        }
    }

    let default_path = PathBuf::from(format!("{}/{}-{}", addr_dir, display_num, "unix-original"));
    if default_path.is_file() {
        return Ok(default_path);
    }

    Err(Error::Address(format!(
        "IBus address file not found in {} for display :{}",
        addr_dir, display_num
    )))
}

/// Parse a unix D-Bus address to extract the socket path.
/// E.g. "unix:path=/tmp/socket,guid=abc" -> Some("/tmp/socket")
/// E.g. "unix:abstract=ibus,guid=abc" -> Some("@ibus")
fn parse_unix_socket_path(address: &str) -> Option<String> {
    if let Some(params) = address.strip_prefix("unix:") {
        for param in params.split(',') {
            if let Some(path) = param.strip_prefix("path=") {
                return Some(path.to_owned());
            }
            if let Some(abstract_path) = param.strip_prefix("abstract=") {
                return Some(format!("@{}", abstract_path));
            }
        }
        // Fallback for cases where "unix:/path" is directly specified
        if !params.contains('=') && !params.is_empty() {
            return Some(params.to_owned());
        }
    }
    None
}

fn parse_address(contents: &str) -> Result<Address> {
    let mut host = String::new();
    let mut port = 0u32;
    let mut socket_path = String::new();

    for line in contents.lines() {
        let line = line.trim();
        let val = if let Some(v) = line.strip_prefix("--host=") {
            Some(("host", v))
        } else if let Some(v) = line.strip_prefix("--port=") {
            Some(("port", v))
        } else if let Some(v) = line.strip_prefix("--address=") {
            Some(("address", v))
        } else if let Some(v) = line.strip_prefix("IBUS_ADDRESS=") {
            Some(("address", v))
        } else {
            None
        };

        if let Some((key, value)) = val {
            if key == "host" {
                host = value.to_owned();
            } else if key == "port" {
                port = value
                    .parse()
                    .map_err(|e| Error::Address(format!("Invalid port: {}", e)))?;
            } else if key == "address" {
                if value.starts_with("unix:") {
                    if let Some(parsed_path) = parse_unix_socket_path(value) {
                        socket_path = parsed_path;
                    }
                } else {
                    socket_path = value.to_owned();
                }
            }
        }
    }

    if socket_path.is_empty() && port == 0 && host.is_empty() {
        return Err(Error::Address(
            "No valid address found in IBus address file".into(),
        ));
    }

    validate_host(&host)?;
    validate_socket_path(&socket_path)?;

    Ok(Address {
        host,
        port,
        socket_path,
    })
}

fn validate_host(host: &str) -> Result<()> {
    if host.is_empty() {
        return Ok(());
    }
    // Allow only safe characters: alphanumeric, '.', '-', ':', '[', ']'
    if host
        .chars()
        .all(|c| c.is_alphanumeric() || c == '.' || c == '-' || c == ':' || c == '[' || c == ']')
    {
        Ok(())
    } else {
        Err(Error::Address(format!(
            "Invalid characters in host: {}",
            host
        )))
    }
}

fn validate_socket_path(path: &str) -> Result<()> {
    if path.is_empty() {
        return Ok(());
    }
    // Reject commas as they are used as option delimiters in D-Bus addresses
    if path.contains(',') {
        return Err(Error::Address(
            "Socket path contains invalid character ','".into(),
        ));
    }
    Ok(())
}

pub fn connect_address() -> Result<String> {
    let addr = read_ibus_address(None)?;
    if !addr.socket_path.is_empty() {
        if addr.socket_path.starts_with('@') {
            Ok(format!("unix:abstract={}", &addr.socket_path[1..]))
        } else {
            Ok(format!("unix:path={}", addr.socket_path))
        }
    } else {
        Ok(format!("tcp:host={},port={}", addr.host, addr.port))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_address_valid_unix_with_options() {
        let content = "--address=unix:path=/tmp/socket,guid=abcdef,param=value\n";
        let addr = parse_address(content).unwrap();
        assert_eq!(addr.socket_path, "/tmp/socket");
    }

    #[test]
    fn test_parse_address_unix_guid_first() {
        let content = "--address=unix:guid=abcdef,path=/tmp/socket,param=value\n";
        let addr = parse_address(content).unwrap();
        assert_eq!(addr.socket_path, "/tmp/socket");
    }

    #[test]
    fn test_parse_address_unix_abstract() {
        let content = "--address=unix:abstract=ibus-websocket,guid=abcdef\n";
        let addr = parse_address(content).unwrap();
        assert_eq!(addr.socket_path, "@ibus-websocket");
    }

    #[test]
    fn test_parse_address_unix_direct() {
        let content = "--address=unix:/tmp/socket\n";
        let addr = parse_address(content).unwrap();
        assert_eq!(addr.socket_path, "/tmp/socket");
    }

    #[test]
    fn test_parse_address_valid_unix() {
        let content = "--address=unix:path=/tmp/ibus-socket\n";
        let addr = parse_address(content).unwrap();
        assert_eq!(addr.socket_path, "/tmp/ibus-socket");
        assert!(addr.host.is_empty());
        assert_eq!(addr.port, 0);
    }

    #[test]
    fn test_parse_address_valid_tcp() {
        let content = "--host=127.0.0.1\n--port=1234\n";
        let addr = parse_address(content).unwrap();
        assert_eq!(addr.host, "127.0.0.1");
        assert_eq!(addr.port, 1234);
        assert!(addr.socket_path.is_empty());
    }

    #[test]
    fn test_parse_address_invalid_host() {
        let content = "--host=127.0.0.1,param=value\n--port=1234\n";
        assert!(parse_address(content).is_err());
    }

    #[test]
    fn test_parse_address_invalid_socket_path() {
        let content = "--address=unix:guid=abcdef\n";
        assert!(parse_address(content).is_err());
    }
}
