/*
 *  DVBTune — for generating a DVBv5 channels file from a DVBv5 transmitter file.
 *
 *  Copyright © 2019, 2020  Russel Winder
 *
 *  This program is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with this program. If not, see <http://www.gnu.org/licenses/>.
 */

use std::ffi::{CStr,CString};
use std::io::{Write, stderr};
use std::thread::sleep;
use std::time::Duration;

use libc;

use dvbv5_sys;

use dvbv5::types::{DmxFd, FilePtr, FrontendId, FrontendParametersPtr, ScanHandlerPtr};

#[derive(Debug)]
pub struct ChannelsData {
    ptr: FilePtr,
}

impl ChannelsData {
    pub fn new_from_str(channels_data_path:&str) -> Result<ChannelsData, ()> {
        match FilePtr::new_from_path(channels_data_path, dvbv5_sys::fe_delivery_system_SYS_UNDEFINED, dvbv5_sys::dvb_file_formats_FILE_DVBV5) {
            Ok(ptr) => Ok(ChannelsData { ptr }),
            Err(_) => Err(()),
        }
    }
    pub fn new_from_fileptr(ptr: FilePtr) -> ChannelsData {
        ChannelsData{ptr}
    }
    pub fn write(&self, output_path: &str, fei: &FrontendId) -> bool {
        if self.ptr.get_c_ptr().is_null() {
            panic!("ChannelData instance not properly initialised.");
        } else {
            match FrontendParametersPtr::new(fei, None, None) {
                Ok(frontend_parameters) => {
                    unsafe {
                        if dvbv5_sys::dvb_write_file_format(CString::new(output_path).unwrap().as_ptr(), self.ptr.get_c_ptr(), (*frontend_parameters.get_c_ptr()).current_sys, dvbv5_sys::dvb_file_formats_FILE_DVBV5) == 0 {
                            frontend_parameters.log(dvbv5_sys::LOG_INFO, &format!("\nWrote virtual channels file to: {}", output_path));
                            true
                        } else {
                            frontend_parameters.log(dvbv5_sys::LOG_INFO, &format!("\nWrite to {} failed.", output_path));
                            false
                        }
                    }
                },
                Err(_) => panic!("Could not create FrontendParametersPtr instance."),
            }
        }
    }
}

#[derive(Debug)]
pub struct TransmitterData {
    ptr: FilePtr,
}

impl TransmitterData {
    pub fn new(transmitter_file_path: &str) -> Result<TransmitterData, ()> {
        match FilePtr::new_from_path(transmitter_file_path, dvbv5_sys::fe_delivery_system_SYS_UNDEFINED, dvbv5_sys::dvb_file_formats_FILE_DVBV5) {
            Ok(ptr) => Ok(TransmitterData { ptr }),
            Err(_) => Err(()),
        }
    }

    /// Write scan status data to stderr. If stderr is a terminal then use the codes to change colours.
    extern "C" fn frontend_check(_arguments: *mut std::ffi::c_void, frontend_parameters: *mut dvbv5_sys::dvb_v5_fe_parms) -> i32 {
        unsafe {
            let logger = (*frontend_parameters).logfunc.unwrap();
            let mut status = dvbv5_sys::fe_status_FE_NONE;
            let mut n_status_lines = 0 as u32;
            let mut stream = stderr();
            let  stream_fd = libc::STDERR_FILENO;
            // Try 20 times to get a lock. It usually takes three to six attempts.
            for _ in 0..20 {
                if (*frontend_parameters).abort != 0 { return 0; }
                if dvbv5_sys::dvb_fe_get_stats(frontend_parameters) != 0 {
                    logger(dvbv5_sys::LOG_INFO as i32, CString::new("dvb_fe_get_stats failed.").unwrap().as_ptr());
                } else {
                    if dvbv5_sys::dvb_fe_retrieve_stats(frontend_parameters, dvbv5_sys::DTV_STATUS, &mut status) != 0 {
                        logger(dvbv5_sys::LOG_INFO as i32, CString::new("dvb_fe_retrieve_stats of DVT_STATUS failed.").unwrap().as_ptr());
                        status = dvbv5_sys::fe_status_FE_NONE;
                    } else {
                        if libc::isatty(stream_fd) != 0 {
                            if n_status_lines != 0 {
                                // If there are status lines then return to the beginning of the line go back up a line and clear it.
                                write!(stream, "\r\x1b[{}A\x1b[J", n_status_lines).unwrap();
                                n_status_lines = 0;
                            }
                            if status & dvbv5_sys::fe_status_FE_HAS_LOCK != 0 {
                                // Set colour to bold green
                                write!(stream, "\x1b[1;32m").unwrap();
                            } else {
                                // Set colour to yellow
                                write!(stream, "\x1b[33m").unwrap();
                            }
                        }
                        let buffer_size = 1024;
                        let mut buffer = ['a'; 1024]; // Can't use the "variable".
                        let mut current_position: *mut i8 = buffer.as_ptr() as *mut i8;
                        let mut usable_length = buffer_size;
                        //
                        // TODO dvb_fe_snprintf_stat returns the number of characters "printed", and a negative value on error.
                        //  Should the code be testing that there are no errors?
                        //
                        let mut show = 0;
                        dvbv5_sys::dvb_fe_snprintf_stat(frontend_parameters, dvbv5_sys::DTV_STATUS, 0 as *mut i8, 0, &mut current_position, &mut usable_length, &mut show);
                        for i in 0..dvbv5_sys::MAX_DTV_STATS as i32 {
                            show = 1;
                            dvbv5_sys::dvb_fe_snprintf_stat(frontend_parameters, dvbv5_sys::DTV_QUALITY, CString::new("Quality").unwrap().as_ptr() as *mut i8, i, &mut current_position, &mut usable_length, &mut show);
                            dvbv5_sys::dvb_fe_snprintf_stat(frontend_parameters, dvbv5_sys::DTV_STAT_SIGNAL_STRENGTH, CString::new("Signal").unwrap().as_ptr() as *mut i8, i, &mut current_position, &mut usable_length, &mut show);
                            dvbv5_sys::dvb_fe_snprintf_stat(frontend_parameters, dvbv5_sys::DTV_STAT_CNR, CString::new("C/N").unwrap().as_ptr() as *mut i8, i, &mut current_position, &mut usable_length, &mut show);
                            dvbv5_sys::dvb_fe_snprintf_stat(frontend_parameters, dvbv5_sys::DTV_STAT_ERROR_BLOCK_COUNT, CString::new("UCB").unwrap().as_ptr() as *mut i8, i, &mut current_position, &mut usable_length, &mut show);
                            dvbv5_sys::dvb_fe_snprintf_stat(frontend_parameters, dvbv5_sys::DTV_BER, CString::new("postBER").unwrap().as_ptr() as *mut i8, i, &mut current_position, &mut usable_length, &mut show);
                            dvbv5_sys::dvb_fe_snprintf_stat(frontend_parameters, dvbv5_sys::DTV_PRE_BER, CString::new("preBER").unwrap().as_ptr() as *mut i8, i, &mut current_position, &mut usable_length, &mut show);
                            dvbv5_sys::dvb_fe_snprintf_stat(frontend_parameters, dvbv5_sys::DTV_PER, CString::new("PER").unwrap().as_ptr() as *mut i8, i, &mut current_position, &mut usable_length, &mut show);
                            if current_position != buffer.as_ptr() as *mut i8 {
                                let line = CStr::from_ptr(buffer.as_ptr() as *mut i8).to_str().unwrap();
                                if n_status_lines != 0 {
                                    write!(stream, "\t{}\n", line).unwrap();
                                } else {
                                    write!(stream, "{}\n", line).unwrap();
                                }
                                n_status_lines += 1;
                                current_position = buffer.as_ptr() as *mut i8;
                                usable_length = buffer_size;
                            }
                        }
                        stream.flush().unwrap();
                    }
                }
                if status & dvbv5_sys::fe_status_FE_HAS_LOCK != 0 { break; }
                sleep(Duration::from_millis(100));
            }
            if libc::isatty(stream_fd) != 0 {
                //  Set default foreground colour without bold.
                write!(stream, "\x1b[22;39m").unwrap();
                stream.flush().unwrap();
            }
            if status & dvbv5_sys::fe_status_FE_HAS_LOCK != 0 { 0 } else { -1 }
        }
    }

    pub fn scan(&self, frontend_id: &FrontendId) -> ChannelsData {
        // TODO These should be defaulted parameters.
        let other_nit = 0;
        let timeout_multiplier = 1;
        let get_detected = 1;
        let get_nit = 1;
        let dont_add_new_frequencies = false;
        //
        // TODO Must do better error handling here.
        let frontend_parameters = FrontendParametersPtr::new(&frontend_id,None, None).unwrap();
        let dmx_fd = DmxFd::new(&frontend_id).unwrap();
        let mut channels_file: *mut dvbv5_sys::dvb_file = 0 as *mut dvbv5_sys::dvb_file;
        let mut count = 1;
        unsafe {
            let mut entry = (*self.ptr.get_c_ptr()).first_entry;
            while ! entry.is_null() {
                let mut frequency: u32 = 0;
                let rc = dvbv5_sys::dvb_retrieve_entry_prop(entry, dvbv5_sys::DTV_FREQUENCY, &mut frequency as *mut u32);
                assert_eq!(rc, 0);
                frontend_parameters.log(dvbv5_sys::LOG_INFO, &format!("\nScanning frequency #{} {}", count, frequency));
                if ! (*entry).channel.is_null() {
                    frontend_parameters.log(dvbv5_sys::LOG_INFO, &format!("Channel Name: {}", CStr::from_ptr((*entry).channel).to_str().unwrap()));
                }
                if ! (*entry).vchannel.is_null() {
                    frontend_parameters.log(dvbv5_sys::LOG_INFO, &format!("Channel number: {}", CStr::from_ptr((*entry).vchannel).to_str().unwrap()));
                }
                if ! (*entry).location.is_null() {
                    frontend_parameters.log(dvbv5_sys::LOG_INFO, &format!("Channel location: {}", CStr::from_ptr((*entry).location).to_str().unwrap()));
                }
                match ScanHandlerPtr::new(&frontend_parameters, entry, &dmx_fd, Some(Self::frontend_check), other_nit, timeout_multiplier) {
                    Ok(scan_handler) => {
                        if (*frontend_parameters.get_c_ptr()).abort != 0 { break; }
                        if dvbv5_sys::dvb_store_channel(&mut channels_file, frontend_parameters.get_c_ptr(), scan_handler.get_c_ptr(), get_detected, get_nit) != 0 {
                            frontend_parameters.log(dvbv5_sys::LOG_INFO, "Failed to store some channels.");
                        }
                        if ! dont_add_new_frequencies {
                            dvbv5_sys::dvb_add_scaned_transponders(frontend_parameters.get_c_ptr(), scan_handler.get_c_ptr(), (*self.ptr.get_c_ptr()).first_entry, entry);
                        }
                    },
                    Err(_) => frontend_parameters.log(dvbv5_sys::LOG_INFO, "Failed to initialise scan handler."),
                }
                entry = (*entry).next;
                count += 1;
            }
        }
        ChannelsData::new_from_fileptr(FilePtr::new_from_dvb_file_ptr(channels_file))
    }
}
