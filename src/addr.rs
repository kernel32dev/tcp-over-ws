use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};

pub fn parse_many_socket_addr(text: &str) -> Vec<SocketAddr> {
    text.split(|x| matches!(x, ';' | ',' | ' '))
        .flat_map(parse_socket_addr)
        .flatten()
        .collect::<Vec<_>>()
}
pub fn parse_one_socket_addr(text: &str) -> Option<SocketAddr> {
    parse_socket_addr(text)[0]
}

fn parse_socket_addr(text: &str) -> [Option<SocketAddr>; 2] {
    let text = text.trim();
    if text.is_empty() {
        return [None, None];
    }
    if let Some(port) = text.parse::<u16>().ok() {
        [
            Some(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, port))),
            Some(SocketAddr::V6(SocketAddrV6::new(
                Ipv6Addr::LOCALHOST,
                port,
                0,
                0,
            ))),
        ]
    } else {
        [
            text.parse()
                .map_err(|_| {
                    println!("o endereço {text:?} não é válido");
                })
                .ok(),
            None,
        ]
    }
}
