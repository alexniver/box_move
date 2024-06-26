use std::net::{IpAddr, Ipv4Addr};

pub(crate) const SERVER_HOST: &str = "127.0.0.1";
pub(crate) const SERVER_PORT: u16 = 6000;
pub(crate) const LOCAL_BIND_IP: IpAddr = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));
