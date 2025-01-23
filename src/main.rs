#![allow(dead_code)]
use std::net::SocketAddr;

mod udp;
mod mm;

fn main() -> std::io::Result<()> {
    let addr: SocketAddr = "127.0.0.1:9000".parse().expect("unable to parse address");

    let mut reader = udp::UdpReader::new();
    let _recv_bytes = udp::connect(&reader.socket, addr).expect("cannot connect to ACC");

    //setup memory mapping
    let memory_map = mm::MMReader::new(); 

    println!("connected!");
    let mut _counter = 0;
    loop {
        // grab UDP data
        /* 
        reader.listen().unwrap();
        match InboundMessageType::try_from(reader.read_u8().unwrap()).unwrap() {
            InboundMessageType::RegistrationResult => {
                let registration = parse_registration_result(&mut reader).unwrap();
                println!("{:#?}", registration);
                request_entry_list(&reader.socket, registration.connection_id).unwrap();
                request_track_data(&reader.socket, registration.connection_id).unwrap();
            }
            InboundMessageType::RealtimeUpdate => {
                /*
                println!("realtime update");
                let realtime_update = parse_realtime_update(&mut reader).unwrap();
                println!("{:#?}", realtime_update);
                */
            }
            InboundMessageType::RealtimeCarUpdate => {
                /*
                if counter % 1000000000 == 0 {
                    println!("realtime car update");
                    let realtime_update = parse_realtime_car_update(&mut reader).unwrap();
                    println!("{:#?}", realtime_update);
                    // @TODO we can update car/driver entry list here
                    counter = 0;
                } else {
                    counter += 1;
                }
                */
            }
            InboundMessageType::EntryList => {
                /*
                println!("entry list");
                let entries = parse_entry_list(&mut reader).unwrap();
                println!("{:#?}", entries);
                */
            }
            InboundMessageType::EntryListCar => {
                /*
                println!("entry list car");
                let car_list = parse_entry_list_car(&mut reader).unwrap();
                println!("{:#?}", car_list);
                */
            }
            InboundMessageType::TrackData => {
                println!("track data");
                let track_data = parse_track_data(&mut reader).unwrap();
                println!("{:#?}", track_data);
            }
            InboundMessageType::BroadcastingEvent => {
                println!("broadcasting event");
                let broadcast = parse_broadcasting_event(&mut reader).unwrap();
                println!("{:#?}", broadcast)
            }
        }
        */

        // grab shared memory data
        // CHECK FOR NEW PACKETS FIRST??
        println!("struct: {:#?}", memory_map.get_physics().packet_id);
    }
}

