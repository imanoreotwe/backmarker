#![allow(dead_code)]
use std::{
    cell::{Ref, RefCell},
    collections::{HashMap, VecDeque},
    mem::drop,
    net::SocketAddr,
    sync::Arc,
    time::{Duration, Instant},
};

use iced::{
    futures::{SinkExt, Stream},
    stream,
    widget::{column, container, row, text, Column, Row},
    window::{self, Settings},
    Element,
    Length::Fill,
    Result, Subscription, Task,
};

use log::{debug, error, info, trace};

mod mm;
mod udp;
mod utils;

#[derive(Debug)]
struct Car {
    car_info: udp::CarInfo,
    laps: Vec<udp::LapInfo>,
    update_ready: bool,
    lap_count: u16,
    position: u16,
    prev: Option<u16>,
    next: Option<u16>,
}

struct Backmarker {
    /// Maps car index to `Car` struct
    cars: HashMap<u16, RefCell<Car>>,
    /// Car index of the leader
    leader: Option<u16>,
    last: Option<u16>,
    update_queue: Vec<u16>,
}

#[derive(Debug)]
enum Message {
    Tick(Instant),
    RealTimeCarUpdate(udp::RealtimeCarUpdate),
    EntryList(udp::EntryList),
    CarInfo(udp::CarInfo),
    BroadcastingEvent(udp::BroadcastingEvent),
}

fn main() -> Result {
    env_logger::init();
    info!("backmarker started");
    iced::daemon("backmarker", Backmarker::update, Backmarker::view)
        .subscription(Backmarker::subscription)
        .run_with(move || Backmarker::new())
}

impl Backmarker {
    fn new() -> (Backmarker, Task<Message>) {
        info!("starting ui");
        let bm = Backmarker {
            cars: HashMap::new(),
            leader: None,
            last: None,
            update_queue: vec![],
        };

        let (_main_window_id, open_main_window) = window::open(Settings::default());

        (bm, open_main_window.then(|_| Task::none()))
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick(_now) => Task::none(),
            Message::RealTimeCarUpdate(realtime_update) => {
                trace!("realtime update message");
                if self.cars.contains_key(&realtime_update.car_index)
                    && self.update_queue.contains(&realtime_update.car_index)
                {
                    let queue_index = self
                        .update_queue
                        .binary_search(&realtime_update.car_index)
                        .unwrap();
                    self.update_queue.remove(queue_index);
                    {
                        let mut car = self
                            .cars
                            .get(&realtime_update.car_index)
                            .unwrap()
                            .borrow_mut();

                        car.laps.push(realtime_update.last_lap);
                        car.update_ready = false;
                        car.lap_count = realtime_update.laps;

                        if car.position != realtime_update.position {
                            if realtime_update.car_index == car.car_info.car_index {
                                car.position = realtime_update.position
                            } else {
                                self.cars
                                    .get(&realtime_update.car_index)
                                    .unwrap()
                                    .borrow_mut()
                                    .position = realtime_update.position;
                            }
                        }
                    }

                    let car = self.cars.get(&realtime_update.car_index).unwrap().borrow();

                    if car.position != realtime_update.position {
                        // remove step
                        if car.prev.is_some() {
                            let prev = car.prev.unwrap();
                            self.cars.get(&prev).unwrap().borrow_mut().next = car.next;
                            if car.next.is_some() {
                                // prev <- next
                                self.cars.get(&car.next.unwrap()).unwrap().borrow_mut().prev =
                                    car.prev;
                            }
                        }

                        // add step
                        let mut inserting_car = self
                            .cars
                            .get(&self.find_position_index_or(realtime_update.position))
                            .unwrap()
                            .borrow_mut();
                        if inserting_car.prev.is_some() {
                            self.cars
                                .get(&inserting_car.prev.unwrap())
                                .unwrap()
                                .borrow_mut()
                                .next = Some(car.car_info.car_index);

                            inserting_car.prev = Some(car.car_info.car_index);
                        } else {
                            self.leader = Some(car.car_info.car_index);
                        }
                    }
                }
                Task::none()
            }
            Message::CarInfo(car_info) => {
                trace!("car info message");
                let index = car_info.car_index;
                if !self.cars.contains_key(&index) {
                    self.cars.insert(
                        car_info.car_index.into(),
                        RefCell::new(Car {
                            car_info,
                            laps: vec![],
                            update_ready: false,
                            lap_count: 0,
                            position: 0,
                            prev: self.last,
                            next: None,
                        }),
                    );

                    if self.leader.is_none() {
                        self.leader = Some(index);
                    } else {
                        let mut curr = self.cars.get(&self.last.unwrap()).unwrap().borrow_mut();
                        curr.next = Some(index);
                        curr.prev = self.last;
                    }
                    self.last = Some(index);
                }
                Task::none()
            }
            Message::BroadcastingEvent(broadcast) => {
                trace!("broadcast event message");
                match broadcast.event_type {
                    udp::BroadcastingEventType::LapCompleted => {
                        self.update_queue.push(broadcast.car_id as u16);
                    }
                    _ => {}
                }
                Task::none()
            }
            Message::EntryList(entry_list) => Task::none(),
            _ => Task::none(),
        }
    }

    fn view(&self, _id: window::Id) -> Element<Message> {
        trace!("rendering!");
        debug! {"cars: {:#?}", self.cars};
        let mut col_vec: Vec<Element<'_, _, _, _>> = vec![];

        if self.leader.is_some() {
            let mut car = self.cars.get(&self.leader.unwrap());
            while car.is_some() {
                // temporary
                let laptime = if car.unwrap().borrow().laps.last().is_none() {
                    0
                } else {
                    car.unwrap().borrow().laps.last().unwrap().laptime_ms
                };
                col_vec.push(
                    container(
                        row![
                            text(car.unwrap().borrow().position),
                            text(car.unwrap().borrow().car_info.race_number),
                            text(utils::ms_to_string(laptime))
                        ]
                        .spacing(4),
                    )
                    .into(),
                );
                if car.unwrap().borrow().next.is_none() {
                    break;
                }
                car = self.cars.get(&car.unwrap().borrow().next.unwrap());
            }
        }
        container(Column::from_vec(col_vec))
            .center_x(Fill)
            .center_y(Fill)
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        let tick = iced::time::every(Duration::from_millis(100)).map(Message::Tick);
        let udp_sub = Subscription::run(udp_worker);
        Subscription::batch(vec![tick, udp_sub])
    }

    /// finds the car index at `position` on track or the last car
    fn find_position_index_or(&self, position: u16) -> u16 {
        let mut curr = self.cars.get(&self.leader.unwrap()).unwrap().borrow();
        loop {
            if curr.position == position {
                return curr.car_info.car_index;
            }
            if curr.next.is_some() {
                curr = self.cars.get(&curr.next.unwrap()).unwrap().borrow();
            } else {
                return curr.car_info.car_index;
            }
        }
    }
}

fn udp_worker() -> impl Stream<Item = Message> {
    stream::channel(100, |mut output| async move {
        let addr: SocketAddr = "127.0.0.1:9000".parse().expect("unable to parse address");
        let mut reader = udp::UdpReader::new();

        let _recv_bytes = udp::connect(&reader.socket, addr).expect("cannot connect to ACC");
        //setup memory mapping
        //let memory_map = mm::MMReader::new();

        loop {
            // grab UDP data
            reader.listen().unwrap(); // could be droping packets here
            match udp::InboundMessageType::try_from(reader.read_u8().unwrap()).unwrap() {
                udp::InboundMessageType::RegistrationResult => {
                    let registration = udp::parse_registration_result(&mut reader).unwrap();
                    info!("connected to acc!");
                    trace!("{:#?}", registration);
                    udp::request_entry_list(&reader.socket, registration.connection_id)
                        .expect("could not send entrylist request");
                    udp::request_track_data(&reader.socket, registration.connection_id)
                        .expect("could not send trackdata request");
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
                    trace!("got RealtimeCarUpdate!");
                    output
                        .send(Message::RealTimeCarUpdate(realtime_update))
                        .await
                        .expect("could not send message");
                }
                udp::InboundMessageType::EntryList => {
                    let entries = udp::parse_entry_list(&mut reader).unwrap();
                    trace!("got entry list!");
                    output
                        .send(Message::EntryList(entries))
                        .await
                        .expect("could not send message");
                }
                udp::InboundMessageType::EntryListCar => {
                    let car_info = udp::parse_entry_list_car(&mut reader).unwrap();
                    trace!("got car info!");
                    output
                        .send(Message::CarInfo(car_info))
                        .await
                        .expect("could not send message");
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
                    trace!("got broadcasting event!");
                    output
                        .send(Message::BroadcastingEvent(broadcast))
                        .await
                        .expect("could not send message");
                }
            }
            // grab shared memory data
            // CHECK FOR NEW PACKETS FIRST??
            //println!("struct: {:#?}", memory_map.get_physics().packet_id);
        }
    })
}
