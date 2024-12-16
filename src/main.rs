#![allow(dead_code)]
use std::{
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

#[derive(Debug)]
struct DriverInfo{
    name: String
}

#[derive(Debug)]
struct CarInfo {
    car_index: usize,
    car_model_type: u8,
    team_name: String,
    race_number: u16,
    cup_category: u8,
    current_driver_index: usize,
    drivers: Vec<DriverInfo>,
    nationality: String // maybe enum
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

}

#[derive(Debug)]
enum InboundMessage {
    RegistrationResult(RegistrationResult),
    EntryList(EntryList),
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

struct UdpReader {
    buf: [u8; 1500],
    size: usize,
    pointer: usize,
    socket: UdpSocket,
}

impl UdpReader {
    fn new() -> Self {
        UdpReader {
            buf: [0; 1500],
            size: 0,
            pointer: 0,
            socket: UdpSocket::bind("127.0.0.1:0").expect("unable to bind to UDP socket"),
        }
    }

    fn listen(&mut self) -> Result<(), String> {
        self.size = self
            .socket
            .recv(&mut self.buf)
            .expect("could not read socket");
        self.pointer = 0;
        //println!("{:?}", self.buf);
        Ok(())
    }

    fn read_bytes(&mut self, count: usize) -> Result<Vec<u8>, String> {
        Ok(self.buf[self.pointer..self.pointer+count].to_vec()).and_then(|result| {
            self.pointer += count;
            Ok(result)
        })
    }

    fn read_string(&mut self) -> Result<String, String> {
        let size = u16::from_le_bytes(self.buf[self.pointer..self.pointer + 1].try_into().unwrap());
        self.pointer += 2;
        let end = self.pointer + size as usize;
        match core::str::from_utf8(&self.buf[self.pointer..end]) {
            Ok(s) => Ok(s.to_owned()),
            Err(_e) => Err("could not parse string".to_string()),
        }
    }
}

fn main() -> std::io::Result<()> {
    let addr: SocketAddr = "127.0.0.1:9000".parse().expect("unable to parse address");

    let mut reader = UdpReader::new();
    let _recv_bytes = connect(&reader.socket, addr).expect("cannot connect to ACC");
    println!("connected!");
    loop {
        reader.listen().unwrap();
        let buf = reader.read_bytes(1).unwrap();
        match InboundMessageType::try_from(buf[0]).unwrap() {
            InboundMessageType::RegistrationResult => {
                let InboundMessage::RegistrationResult(registration) = parse_registration_result(&mut reader).unwrap();
                request_entry_list(&reader.socket, registration.connection_id).unwrap();
                request_track_data(&reader.socket, registration.connection_id).unwrap();
                return Ok(())
            },
            InboundMessageType::RealtimeUpdate => {
                parse_realtime_update().unwrap();
                return Ok(())
            },
            InboundMessageType::RealtimeCarUpdate => {
                parse_realtime_car_update().unwrap();
                return Ok(())
            },
            InboundMessageType::EntryList => {
                parse_entry_list().unwrap();
                return Ok(())
            },
            InboundMessageType::EntryListCar => {
                parse_entry_list_car().unwrap();
                return Ok(())
            },
            InboundMessageType::TrackData => {
                parse_track_data().unwrap();
                return Ok(())
            },
            InboundMessageType::BroadcastingEvent => {
                parse_broadcasting_event().unwrap();
                return Ok(())
            },
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
    buf.extend_from_slice(&5u16.to_le_bytes());
    buf.extend_from_slice(b"12345"); // connection password
    buf.extend_from_slice(&100u32.to_le_bytes()); // realtime update interval
    buf.extend_from_slice(&5u16.to_le_bytes());
    buf.extend_from_slice(b"12345"); // command password

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

fn parse_registration_result(reader: &mut UdpReader) -> Result<InboundMessage, String> {
    let buf = reader.read_bytes(8).unwrap();
    println!("{:?}", buf);
    match buf[4] {
        0 => Ok(InboundMessage::RegistrationResult(RegistrationResult {
            connection_id: u32::from_le_bytes(buf[0..4].try_into().unwrap()),
            is_readonly: buf[6] == 1,
        })),
        1 => Err(reader.read_string().unwrap()),
        _ => Err("could not parse message".to_string()),
    }
}

fn parse_realtime_update() -> Result<InboundMessage, String> {
    Err("not implemented".to_string())
}

fn parse_realtime_car_update() -> Result<InboundMessage, String> {
    Err("not implemented".to_string())
}

fn parse_entry_list(reader: &mut UdpReader) -> Result<InboundMessage, String> {
    let
}

fn parse_entry_list_car() -> Result<InboundMessage, String> {
    Err("not implemented".to_string())
}

fn parse_track_data() -> Result<InboundMessage, String> {
    Err("not implemented".to_string())
}

fn parse_broadcasting_event() -> Result<InboundMessage, String> {
    Err("not implemented".to_string())
}
