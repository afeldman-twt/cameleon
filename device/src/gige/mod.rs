/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

pub mod protocol;
pub mod register_map;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

pub const GVCP_DEFAULT_PORT: u16 = 3956;

use std::time;

use tokio::{net, time as tokiotime};

use protocol::{ack, cmd, prelude::*};

use std::net::Ipv4Addr;

fn calc_broadcast(ip: Ipv4Addr, mask: Ipv4Addr) -> Ipv4Addr {
    let ip = u32::from(ip);
    let mask = u32::from(mask);
    Ipv4Addr::from(ip | !mask)
}

pub async fn enumerate_devices(timeout: time::Duration) -> Result<Vec<ack::Discovery>> {
    let interfaces = pnet::datalink::interfaces();
    let packet = cmd::Discovery::new().finalize(0xffff);
    let mut buf = [0_u8; 1024];
    packet.serialize(buf.as_mut()).unwrap();
    let length = packet.length() as usize;

    let mut discoveries = vec![];

    println!("🔍 Sending discovery packets on all interfaces...");

    for iface in interfaces {
        for ip_net in iface.ips {
            match (ip_net.ip(), ip_net.mask()) {
                (std::net::IpAddr::V4(ip), std::net::IpAddr::V4(mask)) => {
                    let broadcast = calc_broadcast(ip, mask);
                    let local_addr = std::net::SocketAddr::new(std::net::IpAddr::V4(ip), 0);

                    let sock = net::UdpSocket::bind(local_addr).await?;
                    sock.set_broadcast(true)?;

                    sock.send_to(&buf[..length], (broadcast, GVCP_DEFAULT_PORT))
                        .await?;
                    println!("📡 Sent discovery from {} to {}", ip, broadcast);

                    if let Ok(Ok((size, _addr))) =
                        tokiotime::timeout(timeout, sock.recv_from(&mut buf)).await
                    {
                        let data = &buf[..size];
                        if let Ok(ack) = ack::AckPacket::parse(&data.to_vec()) {
                            if ack.status().is_success() {
                                if let Ok(discovery) = ack.ack_data_as::<ack::Discovery>() {
                                    println!("🎯 Got Discovery from {}: {:?}", ip, discovery);
                                    discoveries.push(discovery);
                                }
                            }
                        }
                    }
                }
                _ => continue,
            }
        }
    }

    println!("📦 Total discoveries received: {}", discoveries.len());
    Ok(discoveries)
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("io error: {0}")]
    Io(#[from] async_std::io::Error),

    #[error("packet is broken: {0}")]
    InvalidPacket(std::borrow::Cow<'static, str>),

    #[error("invalid data: {0}")]
    InvalidData(std::borrow::Cow<'static, str>),
}
