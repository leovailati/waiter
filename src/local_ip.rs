use std::net::{IpAddr, UdpSocket, Ipv4Addr};
use std::io::{Result, Error, ErrorKind};


const REMOTE_PORT: u16 = 80;


pub fn local_ip() -> Result<IpAddr> {
    let remote_addrs = [Ipv4Addr::new(192, 0, 2, 0),
                        Ipv4Addr::new(198, 51, 100, 0),
                        Ipv4Addr::new(203, 0, 113, 0)];

    let sock = UdpSocket::bind(("0.0.0.0", 0))?;
    let mut ips = Vec::new();

    for &addr in &remote_addrs {
        sock.connect((addr, REMOTE_PORT))?;
        let ip = sock.local_addr()?.ip();

        if ips.contains(&ip) {
            return Ok(ip);
        } else {
            ips.push(ip);
        }
    }


    Err(Error::from(ErrorKind::AddrNotAvailable))
}
