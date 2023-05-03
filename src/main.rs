use pnet::{
    datalink,
    packet::arp::{ArpOperations, ArpPacket},
    packet::ethernet,
    packet::{FromPacket, Packet},
};

fn main() {
    let interface = get_interface();
    let (mut _tx, mut rx) = match datalink::channel(&interface, Default::default()) {
        Ok(datalink::Channel::Ethernet(tx, rx)) => (tx, rx),
        Ok(..) => panic!("Unsupported channel type"),
        Err(..) => panic!("Cannot create channel for {}", interface.name),
    };

    while let Ok(frame) = rx.next() {
        let packet = match ethernet::EthernetPacket::new(frame) {
            Some(packet) => packet,
            None => continue,
        };
        let packet = match packet.get_ethertype() {
            ethernet::EtherTypes::Arp => packet,
            _ => continue,
        };
        let packet = ArpPacket::new(packet.payload()).unwrap();
        let arp = ArpPacket::from_packet(&packet);
        match &arp.operation {
            &ArpOperations::Reply => {
                let sender_mac = &arp.sender_hw_addr;
                let sender_ip = &arp.sender_proto_addr;
                let target_mac = &arp.target_hw_addr;
                let target_ip = &arp.target_proto_addr;
                println!(
                    "{}({}) -> {}({})",
                    sender_ip, sender_mac, target_ip, target_mac
                );
            }
            &ArpOperations::Request => continue,
            _ => unreachable!("Unsupported ARP operation code"),
        }
    }
}

fn get_interface() -> datalink::NetworkInterface {
    datalink::interfaces()
        .into_iter()
        .find(|interface| {
            if interface.mac.is_none() {
                return false;
            }

            if interface.ips.is_empty() || !interface.is_up() || interface.is_loopback() {
                return false;
            }

            if interface.ips.iter().find(|ip| ip.is_ipv4()).is_some() {
                return true;
            };

            false
        })
        .unwrap()
}
