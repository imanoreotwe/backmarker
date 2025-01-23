#![allow(dead_code)]
use std::{
    collections::HashMap,
    net::SocketAddr,
    time::{Duration, Instant},
};

use iced::{
    widget::{column, container, row, text, Column, Row},
    window::{self, Settings},
    Element,
    Length::Fill,
    Result, Subscription, Task,
};

mod mm;
mod udp;

#[derive(Debug)]
struct CarLaps {
    car_info: udp::CarInfo,
    laps: Vec<udp::LapInfo>,
    update_ready: bool,
    lap_count: u16,
}

struct Backmarker {
    cars: HashMap<usize, CarLaps>,
    reader: udp::UdpReader,
}

#[derive(Debug)]
enum Message {
    NewLap(CarLaps),
    Tick(Instant),
}

fn main() -> Result {
    iced::daemon("backmarker", Backmarker::update, Backmarker::view)
        .subscription(Backmarker::subscription)
        .run_with(move || Backmarker::new())
}

impl Backmarker {
    fn new() -> (Backmarker, Task<Message>) {
        let addr: SocketAddr = "127.0.0.1:9000".parse().expect("unable to parse address");

        let bm = Backmarker {
            reader: udp::UdpReader::new(),
            cars: HashMap::new(),
        };
        let _recv_bytes = udp::connect(&bm.reader.socket, addr).expect("cannot connect to ACC");

        //setup memory mapping
        //let memory_map = mm::MMReader::new();

        let (_main_window_id, open_main_window) = window::open(Settings::default());

        (bm, open_main_window.then(|_| Task::none()))
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick(now) => {
                // grab UDP data
                self.reader.listen().unwrap(); // could be droping packets here
                match udp::InboundMessageType::try_from(self.reader.read_u8().unwrap()).unwrap() {
                    udp::InboundMessageType::RegistrationResult => {
                        let registration =
                            udp::parse_registration_result(&mut self.reader).unwrap();
                        println!("connected!");
                        //println!("{:#?}", registration);
                        udp::request_entry_list(&self.reader.socket, registration.connection_id)
                            .unwrap();
                        udp::request_track_data(&self.reader.socket, registration.connection_id)
                            .unwrap();
                    }
                    udp::InboundMessageType::RealtimeUpdate => {
                        /*
                        println!("realtime update");
                        let realtime_update = parse_realtime_update(&mut reader).unwrap();
                        println!("{:#?}", realtime_update);
                        */
                    }
                    udp::InboundMessageType::RealtimeCarUpdate => {
                        let realtime_update =
                            udp::parse_realtime_car_update(&mut self.reader).unwrap();
                        if self
                            .cars
                            .contains_key(&(realtime_update.car_index as usize))
                        {
                            let car = self
                                .cars
                                .get_mut(&(realtime_update.car_index as usize))
                                .unwrap();
                            if car.update_ready {
                                car.laps.push(realtime_update.last_lap);
                                car.update_ready = false;
                                car.lap_count = realtime_update.laps;
                                //println!("{:#?}", car)
                            }
                        }
                    }
                    udp::InboundMessageType::EntryList => {
                        let _entries = udp::parse_entry_list(&mut self.reader).unwrap();
                        println!("got entry list!");
                    }
                    udp::InboundMessageType::EntryListCar => {
                        let car_info = udp::parse_entry_list_car(&mut self.reader).unwrap();
                        self.cars.insert(
                            car_info.car_index.into(),
                            CarLaps {
                                car_info,
                                laps: vec![],
                                update_ready: false,
                                lap_count: 0,
                            },
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
                        let broadcast = udp::parse_broadcasting_event(&mut self.reader).unwrap();
                        match broadcast.event_type {
                            udp::BroadcastingEventType::LapCompleted => {
                                let car = self.cars.get_mut(&(broadcast.car_id as usize));
                                if !car.is_none() {
                                    car.unwrap().update_ready = true;
                                }
                            }
                            _ => {}
                        }
                    }
                }
                // grab shared memory data
                // CHECK FOR NEW PACKETS FIRST??
                //println!("struct: {:#?}", memory_map.get_physics().packet_id);
                Task::none()
            }
            Message::NewLap(update) => Task::none(),
        }
    }

    fn view(&self, id: window::Id) -> Element<Message> {
        let mut col_vec: Vec<Element<'_, _, _, _>> = vec![];
        for car in self.cars.iter() {
            let laptime = if car.1.laps.last().is_none() {
                0
            } else {
                car.1.laps.last().unwrap().laptime_ms
            };
            col_vec.push(
                container(row![text(car.1.car_info.race_number), text(laptime)].spacing(4)).into(),
            )
        }
        container(Column::from_vec(col_vec))
            .center_x(Fill)
            .center_y(Fill)
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        let tick = iced::time::every(Duration::from_millis(100)).map(Message::Tick);
        Subscription::batch(vec![tick])
    }
}
