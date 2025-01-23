#![allow(dead_code)]
use std::{collections::{HashMap, VecDeque}, net::SocketAddr};

mod udp;
mod mm;

#[derive(Debug)]
struct CarLaps {
    car_info: udp::CarInfo,
    laps: Vec<udp::LapInfo>,
    update_ready: bool
}

fn main() -> std::io::Result<()> {
    let addr: SocketAddr = "127.0.0.1:9000".parse().expect("unable to parse address");

    let mut reader = udp::UdpReader::new();
    let _recv_bytes = udp::connect(&reader.socket, addr).expect("cannot connect to ACC");

    //setup memory mapping
    //let memory_map = mm::MMReader::new(); 

    let mut cars: HashMap<usize, CarLaps> = HashMap::new();
    loop {
        // grab UDP data
        reader.listen().unwrap();
        match udp::InboundMessageType::try_from(reader.read_u8().unwrap()).unwrap() {
            udp::InboundMessageType::RegistrationResult => {
                let registration = udp::parse_registration_result(&mut reader).unwrap();
                println!("connected!");
                println!("{:#?}", registration);
                udp::request_entry_list(&reader.socket, registration.connection_id).unwrap();
                udp::request_track_data(&reader.socket, registration.connection_id).unwrap();
            }
            udp::InboundMessageType::RealtimeUpdate => {
                /*
                println!("realtime update");
                let realtime_update = parse_realtime_update(&mut reader).unwrap();
                println!("{:#?}", realtime_update);
                */
            }
            udp::InboundMessageType::RealtimeCarUpdate => {
                let realtime_update = udp::parse_realtime_car_update(&mut reader).unwrap();
                if cars.contains_key(&(realtime_update.car_index as usize)) {
                    let car = cars.get_mut(&(realtime_update.car_index as usize)).unwrap();
                    if car.update_ready {
                        car.laps.push(realtime_update.last_lap);
                        car.update_ready = false;
                        println!("{:#?}", car)
                    }
                } 
            }
            udp::InboundMessageType::EntryList => {
                let _entries = udp::parse_entry_list(&mut reader).unwrap();
                println!("got entry list!");
            }
            udp::InboundMessageType::EntryListCar => {
                let car_info = udp::parse_entry_list_car(&mut reader).unwrap();
                cars.insert(
                    car_info.car_index.into(),
                    CarLaps {
                        car_info,
                        laps: vec![],
                        update_ready: false
                    }
                );
            }
            udp::InboundMessageType::TrackData => {
                /*
                println!("track data");
                let track_data = parse_track_data(&mut reader).unwrap();
                println!("{:#?}", track_data);
                */
            }
            udp::InboundMessageType::BroadcastingEvent => {
                let broadcast = udp::parse_broadcasting_event(&mut reader).unwrap();
                match broadcast.event_type {
                    udp::BroadcastingEventType::LapCompleted => {
                        cars.get_mut(&(broadcast.car_id as usize)).unwrap().update_ready = true;
                    }
                    _ => {}
                }
            }
        }

        // grab shared memory data
        // CHECK FOR NEW PACKETS FIRST??
        //println!("struct: {:#?}", memory_map.get_physics().packet_id);
    }
}

