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
        Ok(_) => panic!("Unsupported channel type"),
        Err(_) => panic!("Cannot create channel for {}", interface.name),
    };

    while let Ok(frame) = rx.next() {
        let ethernet_packet = match ethernet::EthernetPacket::new(frame) {
            Some(packet) if packet.get_ethertype() == ethernet::EtherTypes::Arp => packet,
            _ => continue,
        };
        let arp_packet = ArpPacket::new(ethernet_packet.payload()).unwrap();
        let arp_data = ArpPacket::from_packet(&arp_packet);
        match arp_data.operation {
            ArpOperations::Reply => (),
            ArpOperations::Request => continue,
            _ => panic!("Unsupported ARP operation code"),
        };

        let sender_mac = &arp_data.sender_hw_addr;
        let sender_ip = &arp_data.sender_proto_addr;
        let target_mac = &arp_data.target_hw_addr;
        let target_ip = &arp_data.target_proto_addr;
        println!("{sender_ip}({sender_mac}) -> {target_ip}({target_mac})",);
    }
}

fn get_interface() -> datalink::NetworkInterface {
    datalink::interfaces()
        .into_iter()
        .filter(|interface| !interface.mac.is_none())
        .filter(|interface| interface.is_up() && !interface.is_loopback())
        .filter(|interface| interface.ips.iter().any(|ip| ip.is_ipv4()))
        .next()
        .unwrap()
}
