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
                if name.starts_with(&format!("{}-", display_num)) || name == display_num {
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

fn parse_address(contents: &str) -> Result<Address> {
    let mut host = String::new();
    let mut port = 0u32;
    let mut socket_path = String::new();

    for line in contents.lines() {
        let line = line.trim();
        if let Some(value) = line.strip_prefix("--host=") {
            host = value.to_owned();
        } else if let Some(value) = line.strip_prefix("--port=") {
            port = value
                .parse()
                .map_err(|e| Error::Address(format!("Invalid port: {}", e)))?;
        } else if let Some(value) = line.strip_prefix("--address=") {
            let path = value.to_owned();
            if path.starts_with("unix:") {
                if let Some(socket) = path.strip_prefix("unix:path=") {
                    socket_path = socket.to_owned();
                } else if let Some(socket) = path.strip_prefix("unix:abstract=") {
                    socket_path = format!("@{}", socket);
                } else if let Some(socket) = path.strip_prefix("unix:") {
                    socket_path = socket.to_owned();
                } else {
                    socket_path = path;
                }
            } else {
                socket_path = path;
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
    if host.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '-' || c == ':' || c == '[' || c == ']') {
        Ok(())
    } else {
        Err(Error::Address(format!("Invalid characters in host: {}", host)))
    }
}

fn validate_socket_path(path: &str) -> Result<()> {
    if path.is_empty() {
        return Ok(());
    }
    // Reject commas as they are used as option delimiters in D-Bus addresses
    if path.contains(',') {
        return Err(Error::Address("Socket path contains invalid character ','".into()));
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
        let content = "--address=unix:path=/tmp/socket,param=value\n";
        assert!(parse_address(content).is_err());
    }
}
