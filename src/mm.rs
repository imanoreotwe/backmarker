//! Module for ACC Memory Mapped communication
//! 
//! Reads, Writes, and Serializes data from ACC memory mapped region.
//! Uses unsafe rust because windows.
//! 
//! 

use std::{ffi::{c_void, CString}, mem, ptr};

use windows_sys::{
    Win32::Foundation::*, 
    Win32::System::Memory::*
};

#[derive(Debug)]
#[repr(C)]
pub struct Physics {
    pub packet_id: i32,
    pub gas: f32,
    pub brake: f32,
    pub fuel: f32,
    pub gear: i32,
    pub rpm: i32,
    pub steer_angle: f32,
    pub speed_kmh: f32,
    pub velocity: [f32; 3],
    pub acc_g: [f32; 3], 
    pub wheel_slip: [f32; 4],
    wheel_load: [f32; 4], // unused
    pub wheel_pressure: [f32; 4],
    pub wheel_angular_speed: [f32; 4],
    tire_wear: [f32; 4], // unused
    tyre_dirty_level: [f32; 4], // unused
    pub tyre_core_temp: [f32; 4],   // double field??
    pub camber_rad: [f32; 4],
    pub suspension_travel: [f32; 4],
    pub drs: f32,
    pub tc: f32, // double double field????
    pub heading: f32,
    pub pitch: f32,
    pub roll: f32,
    cg_height: f32, // unused
    pub car_damage: [f32; 5],
    number_of_tyres_out: i32, // unused
    pub pit_limiter_on: i32,
    pub abs: f32, // double double double field?????
    kers_charge: f32, // unused
    kers_input: f32, // unused
    auto_shifter_on: i32,
    ride_height: [f32; 2], // unused
    pub turbo_boost: f32,
    ballast: f32, // unused
    air_density: f32, // unused
    pub air_temp: f32,
    pub road_temp: f32,
    pub local_angular_vel: [f32; 3],
    pub final_ff: f32,
    performance_meter: f32, // unused
    enginer_brake: i32, // unused
    ers_recovery_level: i32, // unused
    ers_power_level: i32, // unused
    ers_heat_charging: i32, // unused
    ers_is_chargin: i32, // unused
    kers_current_kj: f32, // unused
    drs_available: i32, // unused
    drs_enabled: i32, // unused
    pub brake_temp: [f32; 4],
    pub clutch: f32,
    tyre_temp_i: [f32; 4], // unused
    tyre_temp_m: [f32; 4], // unused
    tyre_temp_o: [f32; 4], // unused
    pub is_ai_controlled: i32,
    tyre_contact_point: [f32; 12], // 4x3 array
    tyre_contact_normal: [f32; 12], // 4x3 array
    tyre_contact_heading: [f32; 12], // 4x3 array
    pub brake_bias: f32,
    pub local_velocity: [f32; 3],
    p2p_activation: i32, // unused
    p2p_status: i32, // unused
    current_max_rpm: f32, // unused
    mz: [f32; 4], // unused
    fx: [f32; 4], // unused
    fy: [f32; 4], // unused
    pub slip_ratio: [f32; 4],
    pub slip_angle: [f32; 4],
    tc_in_action: i32, // unused
    abs_in_action: i32, // unused
    suspension_damage: [f32; 4], // unused
    tyre_temp: [f32; 4], // unused
    pub water_temp: f32,
    brake_pressure: [f32; 4], // unused
    pub front_brake_compound: i32,
    pub rear_brake_compound: i32,
    pub pad_life: [f32; 4],
    pub disc_life: [f32; 4],
    pub ignition_on: i32,
    pub starter_engine_on: i32,
    pub is_engine_running: i32,
    pub kerb_vibration: f32,
    pub slip_vibrations: f32,
    pub g_vibrations: f32,
    pub abs_vibrations: f32
}

pub struct MMReader {
    physics_ptr: *const c_void
}

impl MMReader {
    pub fn new() -> Self {
        MMReader {
            physics_ptr: Self::setup_physics().unwrap()
        }
    } 

    fn setup_physics() -> Option<*const c_void> {
        let sz_name= CString::new("Local\\acpmf_physics").unwrap();
        let sz_name_ptr = sz_name.as_ptr() as *const u8;
        unsafe {
            let physics_handle = CreateFileMappingA(
                INVALID_HANDLE_VALUE,
                ptr::null(),
                PAGE_READWRITE,
                0,
                mem::size_of::<Physics>().try_into().unwrap(),
                sz_name_ptr
            )
            .as_mut();

            let memory_map = MapViewOfFile(
                physics_handle.unwrap(),
                FILE_MAP_READ,
                0,
                0,
                mem::size_of::<Physics>().try_into().unwrap(),
            )
            .Value;

            if memory_map.is_null() {
                None
            } else {
                Some(memory_map)
            }
            //let physics_struct = unsafe { & *((map_file_buffer.unwrap() as *const _) as *const Physics) };
        }
        
    }

    pub fn get_physics(&self) -> Physics {
        unsafe {
            let tmp = (self.physics_ptr as *const _) as *const Physics;
            ptr::read(tmp)
        }
    }
}
