#![allow(dead_code)]
use std::{
    collections::HashMap,
    io::Error,
    net::{SocketAddr, UdpSocket},
};

const BROADCASTING_PROTOCOL_VERSION: u8 = 4;

#[repr(u8)]
enum OutboundMessageType {
    RegisterCommand = 1,
    UnregisterCommand = 9,

    RequestEntryList = 10,
    RequestTrackData = 11,

    ChangeHudPage = 49,
    ChangeFocus = 50,
    InstantReplayRequest = 51,

    PlayManualReplayHighlight = 52, // planned?
    SaveManualReplayHighlight = 60, // planned?
}

#[repr(u8)]
enum InboundMessageType {
    RegistrationResult = 1,
    RealtimeUpdate = 2,
    RealtimeCarUpdate = 3,
    EntryList = 4,
    TrackData = 5,
    EntryListCar = 6,
    BroadcastingEvent = 7,
}

impl TryFrom<u8> for InboundMessageType {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(InboundMessageType::RegistrationResult),
            2 => Ok(InboundMessageType::RealtimeUpdate),
            3 => Ok(InboundMessageType::EntryListCar),
            4 => Ok(InboundMessageType::EntryList),
            5 => Ok(InboundMessageType::TrackData),
            6 => Ok(InboundMessageType::EntryListCar),
            7 => Ok(InboundMessageType::BroadcastingEvent),
            _ => Err("could not parse message type"),
        }
    }
}

#[derive(Debug)]
#[repr(u8)]
enum RaceSessionType {
    Practice = 0,
    Qualifying = 4,
    Superpole = 9,
    Race = 10,
    Hotlap = 11,
    Hotstint = 12,
    HotlapSuperpole = 13,
    Replay = 14,
}

impl TryFrom<u8> for RaceSessionType {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(RaceSessionType::Practice),
            4 => Ok(RaceSessionType::Qualifying),
            9 => Ok(RaceSessionType::Superpole),
            10 => Ok(RaceSessionType::Race),
            11 => Ok(RaceSessionType::Hotlap),
            12 => Ok(RaceSessionType::Hotstint),
            13 => Ok(RaceSessionType::HotlapSuperpole),
            14 => Ok(RaceSessionType::Replay),
            _ => Err("could not parse race session type"),
        }
    }
}

#[derive(Debug)]
#[repr(u8)]
enum SessionPhase {
    None = 0,
    Starting = 1,
    PreFormation = 2,
    FormationLap = 3,
    PreSession = 4,
    Session = 5,
    SessionOver = 6,
    PostSession = 7,
    ResultUI = 8,
}

impl TryFrom<u8> for SessionPhase {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(SessionPhase::None),
            1 => Ok(SessionPhase::Starting),
            2 => Ok(SessionPhase::PreFormation),
            3 => Ok(SessionPhase::FormationLap),
            4 => Ok(SessionPhase::PreSession),
            5 => Ok(SessionPhase::Session),
            6 => Ok(SessionPhase::SessionOver),
            7 => Ok(SessionPhase::PostSession),
            8 => Ok(SessionPhase::ResultUI),
            _ => Err("could not parse race session type"),
        }
    }
}

#[derive(Debug)]
struct DriverInfo {
    first_name: String,
    last_name: String,
    short_name: String,
    category: u8, // could potentially be an enum
    nationality: u16,
}

#[derive(Debug)]
struct CarInfo {
    car_index: u16,
    car_model_type: u8,
    team_name: String,
    race_number: u32,
    cup_category: u8,
    current_driver_index: u8,
    drivers: Vec<DriverInfo>,
    nationality: u16, // maybe enum
}

#[derive(Debug)]
enum LapType {
    Outlap,
    Inlap,
    Regular,
}

#[derive(Debug)]
struct LapInfo {
    laptime_ms: u32,
    car_index: u16,
    driver_index: u16,
    lap_splits: Vec<u32>,
    is_invalid: bool,
    is_valid_for_best: bool,
    lap_type: LapType,
}

/// Registration result message
///
/// Message Format:
/// 0-3 : Connection ID
/// 4   : Connection Success
/// 5-6 : Error msg len
/// 7-n : Error msg
#[derive(Debug)]
struct RegistrationResult {
    connection_id: u32,
    is_readonly: bool,
}

/// Entry List message
///
/// Message Format:
/// 0-3 : connection id
/// 4-5 : car count
/// 6-n : car infos
#[derive(Debug)]
struct EntryList {
    connection_id: u32,
    cars: Vec<u16>,
}

#[derive(Debug)]
struct TrackData {
    connection_id: u32,
    track_name: String,
    track_id: u32,
    track_meters: u32,
    camera_sets: HashMap<String, Box<[String]>>,
    hud_pages: Vec<String>,
}

#[derive(Debug)]
struct RealtimeCarUpdate {
    car_index: u16,
    driver_index: u16,
    driver_count: u8,
    gear: u8, // R 0, N 1, 1 2, ...
    world_pos_x: f32,
    world_pos_y: f32,
    yaw: f32,
    car_location: u8, // -, track, pitlane, pit entry pit exit = 4
    kmh: u16,
    position: u16,        // official P/Q/R position (1 indexed)
    cup_position: u16,    // official P/Q/R position (1 indexed)
    track_position: u16,  // position on track (1 based)
    spline_position: f32, // track position between 0.0 and 1.0
    laps: u16,
    delta: u32, // realtime delta to best session lap
    best_session_lap: LapInfo,
    last_lap: LapInfo,
    current_lap: LapInfo,
}

#[derive(Debug)]
struct RealtimeUpdate {
    event_index: u16,
    session_index: u16,
    session_type: RaceSessionType,
    phase: SessionPhase,
    session_time: f32,     //@TODO convert into time struct?
    session_end_time: f32, //@TODO convert into time struct?
    focused_car_index: u32,
    active_camera_set: String,
    active_camera: String,
    current_hud_page: String,
    is_replay_playing: bool,
    replay_session_time: Option<f32>,
    replay_remaining_time: Option<f32>,
    time_of_day: f32, //@TODO convert into time struct?
    ambiant_temp: u8,
    track_temp: u8,
    clouds: f32,
    rain_level: f32,
    wetness: f32,
    best_session_lap: LapInfo,
}

#[derive(Debug)]
enum InboundMessage {
    RegistrationResult(RegistrationResult),
    EntryList(EntryList),
    RealtimeCarUpdate(RealtimeCarUpdate),
    RealtimeUpdate(RealtimeUpdate),
}

struct UdpReader {
    buf: [u8; 65507],
    size: usize,
    pointer: usize,
    socket: UdpSocket,
}

impl UdpReader {
    fn new() -> Self {
        UdpReader {
            buf: [0; 65507],
            size: 0,
            pointer: 0,
            socket: UdpSocket::bind("127.0.0.1:0").expect("unable to bind to UDP socket"),
        }
    }

    fn listen(&mut self) -> Result<usize, String> {
        //println!("{:?}", self.buf);
        self.size = self
            .socket
            .recv(&mut self.buf)
            .expect("could not read socket");
        self.pointer = 0;
        //println!("{:?}", self.buf);
        Ok(self.size)
    }

    fn read_bytes(&mut self, count: usize) -> Result<Vec<u8>, String> {
        Ok(self.buf[self.pointer..self.pointer + count].to_vec()).and_then(|result| {
            self.pointer += count;
            Ok(result)
        })
    }

    fn read_string(&mut self) -> Result<String, String> {
        let size = u16::from_le_bytes(self.read_bytes(2).unwrap().try_into().unwrap());
        match core::str::from_utf8(&self.read_bytes(size as usize).unwrap()) {
            Ok(s) => Ok(s.to_owned()),
            Err(_e) => Err("could not parse string".to_string()),
        }
    }

    fn read_u32(&mut self) -> Result<u32, String> {
        Ok(u32::from_le_bytes(
            self.read_bytes(4).unwrap().try_into().unwrap(),
        ))
    }

    fn read_u16(&mut self) -> Result<u16, String> {
        Ok(u16::from_le_bytes(
            self.read_bytes(2).unwrap().try_into().unwrap(),
        ))
    }

    fn read_u8(&mut self) -> Result<u8, String> {
        Ok(u8::from_le_bytes(
            self.read_bytes(1).unwrap().try_into().unwrap(),
        ))
    }

    fn read_f32(&mut self) -> Result<f32, String> {
        Ok(f32::from_le_bytes(
            self.read_bytes(4).unwrap().try_into().unwrap(),
        ))
    }
}

fn main() -> std::io::Result<()> {
    let addr: SocketAddr = "127.0.0.1:9000".parse().expect("unable to parse address");

    let mut reader = UdpReader::new();
    let _recv_bytes = connect(&reader.socket, addr).expect("cannot connect to ACC");
    println!("connected!");
    loop {
        reader.listen().unwrap();
        match InboundMessageType::try_from(reader.read_u8().unwrap()).unwrap() {
            InboundMessageType::RegistrationResult => {
                let registration = parse_registration_result(&mut reader).unwrap();
                println!("{:#?}", registration);
                request_entry_list(&reader.socket, registration.connection_id).unwrap();
                request_track_data(&reader.socket, registration.connection_id).unwrap();
            }
            InboundMessageType::RealtimeUpdate => {
                //let realtime_update = parse_realtime_update(&mut reader).unwrap();
                //println!("{:#?}", realtime_update);
            }
            InboundMessageType::RealtimeCarUpdate => {
                let realtime_update = parse_realtime_car_update(&mut reader).unwrap();
                println!("{:#?}", realtime_update);
                // @TODO we can update car/driver entry list here
            }
            InboundMessageType::EntryList => {
                println!("entry list");
                let entries = parse_entry_list(&mut reader).unwrap();
                println!("{:#?}", entries);
            }
            InboundMessageType::EntryListCar => {
                //parse_entry_list_car().unwrap();
            }
            InboundMessageType::TrackData => {
                println!("track data");
                let track_data = parse_track_data(&mut reader).unwrap();
                println!("{:#?}", track_data);
            }
            InboundMessageType::BroadcastingEvent => {
                //parse_broadcasting_event().unwrap();
            }
        }
    }
}

fn connect(socket: &UdpSocket, addr: SocketAddr) -> Result<usize, Error> {
    socket.connect(addr).unwrap();
    let mut buf = Vec::with_capacity(26);
    buf.push(OutboundMessageType::RegisterCommand as u8);
    buf.push(BROADCASTING_PROTOCOL_VERSION as u8);
    buf.extend_from_slice(&4u16.to_le_bytes());
    buf.extend_from_slice(b"name"); // display name
    buf.extend_from_slice(&3u16.to_le_bytes());
    buf.extend_from_slice(b"asd"); // connection password
    buf.extend_from_slice(&250u32.to_le_bytes()); // realtime update interval
    buf.extend_from_slice(&0u16.to_le_bytes());
    //buf.extend_from_slice(b""); // command password

    socket.send(&buf)
}

fn disconnect(socket: &UdpSocket) -> Result<usize, Error> {
    let buf = vec![OutboundMessageType::UnregisterCommand as u8];
    socket.send(&buf)
}

fn request_entry_list(socket: &UdpSocket, connection_id: u32) -> Result<usize, Error> {
    let mut buf: Vec<u8> = Vec::with_capacity(5);
    buf.push(OutboundMessageType::RequestEntryList as u8);
    buf.extend_from_slice(&connection_id.to_le_bytes());

    socket.send(&buf)
}

fn request_track_data(socket: &UdpSocket, connection_id: u32) -> Result<usize, Error> {
    let mut buf: Vec<u8> = Vec::with_capacity(5);
    buf.push(OutboundMessageType::RequestTrackData as u8);
    buf.extend_from_slice(&connection_id.to_le_bytes());

    socket.send(&buf)
}

fn parse_registration_result(reader: &mut UdpReader) -> Result<RegistrationResult, String> {
    let connection_id = reader.read_u32().unwrap();
    if reader.read_u8().unwrap() > 0 {
        Ok(RegistrationResult {
            connection_id: connection_id,
            is_readonly: reader.read_u8().unwrap() == 0,
        })
    } else {
        reader.read_u8().unwrap();
        Err(reader.read_string().unwrap())
    }
}

fn parse_lap(reader: &mut UdpReader) -> Result<LapInfo, String> {
    let laptime_ms = reader.read_u32().unwrap();
    let car_index = reader.read_u16().unwrap();
    let driver_index = reader.read_u16().unwrap();

    let split_count = reader.read_u8().unwrap();
    let mut splits: Vec<u32> = vec![];
    for _i in 0..split_count {
        splits.push(reader.read_u32().unwrap());
    }
    let is_invalid = reader.read_u8().unwrap() > 0;
    let is_valid_for_best = reader.read_u8().unwrap() > 0;
    let is_outlap = reader.read_u8().unwrap() > 0;
    let is_inlap = reader.read_u8().unwrap() > 0;

    let lap_type = if is_outlap {
        LapType::Outlap
    } else if is_inlap {
        LapType::Inlap
    } else {
        LapType::Regular
    };

    // a "no" lap may not include a first split
    while splits.len() < 3 {
        splits.push(0);
    }

    Ok(LapInfo {
        laptime_ms,
        car_index,
        driver_index,
        lap_splits: splits,
        is_invalid,
        is_valid_for_best,
        lap_type,
    })
}

fn parse_realtime_update(reader: &mut UdpReader) -> Result<RealtimeUpdate, String> {
    let event_index = reader.read_u16().unwrap();
    let session_index = reader.read_u16().unwrap();
    let session_type = RaceSessionType::try_from(reader.read_u8().unwrap()).unwrap();
    let phase = SessionPhase::try_from(reader.read_u8().unwrap()).unwrap();
    let session_time = reader.read_f32().unwrap();
    let session_end_time = reader.read_f32().unwrap();
    let focused_car_index = reader.read_u32().unwrap();
    let active_camera_set = reader.read_string().unwrap();
    let active_camera = reader.read_string().unwrap();
    let current_hud_page = reader.read_string().unwrap();
    let is_replay_playing = reader.read_u8().unwrap() > 0;
    let mut replay_session_time: Option<f32> = None;
    let mut replay_remaining_time: Option<f32> = None;
    if is_replay_playing {
        replay_session_time = Some(reader.read_f32().unwrap());
        replay_remaining_time = Some(reader.read_f32().unwrap());
    }

    let time_of_day = reader.read_f32().unwrap();
    let ambiant_temp = reader.read_u8().unwrap();
    let track_temp = reader.read_u8().unwrap();
    let clouds = reader.read_u8().unwrap() as f32 / 10.0f32;
    let rain_level = reader.read_u8().unwrap() as f32 / 10.0f32;
    let wetness = reader.read_u8().unwrap() as f32 / 10.0f32;
    let best_session_lap = parse_lap(reader).unwrap();

    Ok(RealtimeUpdate {
        event_index,
        session_index,
        session_type,
        phase,
        session_time,
        session_end_time,
        focused_car_index,
        active_camera_set,
        active_camera,
        current_hud_page,
        is_replay_playing,
        replay_session_time,
        replay_remaining_time,
        time_of_day,
        ambiant_temp,
        track_temp,
        clouds,
        rain_level,
        wetness,
        best_session_lap,
    })
}

fn parse_realtime_car_update(reader: &mut UdpReader) -> Result<RealtimeCarUpdate, String> {
    let car_index = reader.read_u16().unwrap();
    let driver_index = reader.read_u16().unwrap();
    let driver_count = reader.read_u8().unwrap();
    let gear = reader.read_u8().unwrap();
    let world_x = reader.read_f32().unwrap();
    let world_y = reader.read_f32().unwrap();
    let yaw = reader.read_f32().unwrap();
    let car_location = reader.read_u8().unwrap();
    let kmh = reader.read_u16().unwrap();
    let position = reader.read_u16().unwrap();
    let cup_position = reader.read_u16().unwrap();
    let track_position = reader.read_u16().unwrap();
    let spline_position = reader.read_f32().unwrap();
    let laps = reader.read_u16().unwrap();
    let delta = reader.read_u32().unwrap();
    let best_session_lap = parse_lap(reader).unwrap();
    let last_lap = parse_lap(reader).unwrap();
    let current_lap = parse_lap(reader).unwrap();

    Ok(RealtimeCarUpdate {
        car_index,
        driver_index,
        driver_count,
        gear,
        world_pos_x: world_x,
        world_pos_y: world_y,
        yaw,
        car_location,
        kmh,
        position,
        cup_position,
        track_position,
        spline_position,
        laps,
        delta,
        best_session_lap,
        last_lap,
        current_lap,
    })
}

fn parse_entry_list(reader: &mut UdpReader) -> Result<EntryList, String> {
    let connection_id = reader.read_u32().unwrap();
    let car_count = reader.read_u16().unwrap();
    let mut entries = EntryList {
        connection_id: connection_id,
        cars: vec![],
    };

    for _i in 0..car_count {
        let index = u16::from_le_bytes(reader.read_bytes(2).unwrap().try_into().unwrap());
        entries.cars.push(index);
    }

    Ok(entries)
}

fn parse_entry_list_car(reader: &mut UdpReader) -> Result<CarInfo, String> {
    let car_index = reader.read_u16().unwrap();
    let car_model_type = reader.read_u8().unwrap();
    let team_name = reader.read_string().unwrap();
    let race_number = reader.read_u32().unwrap();
    let cup_category = reader.read_u8().unwrap();
    let current_driver_index = reader.read_u8().unwrap();
    let nationality = reader.read_u16().unwrap();

    let driver_count = reader.read_u8().unwrap();
    let mut drivers = Vec::with_capacity(driver_count.into());
    for _i in 0..driver_count {
        let first_name = reader.read_string().unwrap();
        let last_name = reader.read_string().unwrap();
        let short_name = reader.read_string().unwrap();
        let category = reader.read_u8().unwrap();
        let nationality = reader.read_u16().unwrap();

        drivers.push(DriverInfo {
            first_name,
            last_name,
            short_name,
            category,
            nationality,
        });
    }

    Ok(CarInfo {
        car_index,
        car_model_type,
        team_name,
        race_number,
        cup_category,
        current_driver_index,
        drivers,
        nationality,
    })
}

fn parse_track_data(reader: &mut UdpReader) -> Result<TrackData, String> {
    let connection_id = reader.read_u32().unwrap();
    let track_name = reader.read_string().unwrap();
    let track_id = reader.read_u32().unwrap();
    let track_meters = reader.read_u32().unwrap();
    let mut camera_sets = HashMap::new();
    let camera_set_count = reader.read_u8().unwrap();
    for _i in 0..camera_set_count {
        let camera_set_name = reader.read_string().unwrap();
        let camera_count = reader.read_u8().unwrap();

        let mut camera_set = Vec::with_capacity(camera_count.into());
        for _j in 0..camera_count {
            camera_set.push(reader.read_string().unwrap());
        }

        camera_sets.insert(camera_set_name.clone(), camera_set.as_slice().into());
    }

    let hud_pages_count = reader.read_u8().unwrap();
    let mut hud_pages: Vec<String> = Vec::with_capacity(hud_pages_count.into());

    for _i in 0..hud_pages_count {
        hud_pages.push(reader.read_string().unwrap());
    }
    Ok(TrackData {
        connection_id: connection_id,
        track_name: track_name,
        track_id: track_id,
        track_meters: track_meters,
        camera_sets: camera_sets,
        hud_pages: hud_pages,
    })
}

fn parse_broadcasting_event() -> Result<InboundMessage, String> {
    Err("not implemented".to_string())
}
