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

//! A command line application for writing channel files having scanned for
//! channels using a DVB device for a given transmitter file.
//!
//! This is essentially a Rust version of
//! [dvbv5-scan](https://www.linuxtv.org/wiki/index.php/Dvbv5-scan) from
//! [libdvbv5](https://linuxtv.org/docs/libdvbv5/) which is part of the
//! [V4L2 project](https://linuxtv.org/wiki/index.php/V4l-utils) that is part of the
//! [Linux TV](https://linuxtv.org) effort.
//!
//! This application makes use of the [dvbv5 crate](https://gitlab.com/Russel/rust-libdvbv5)
//! which in turn uses the  [dvbv5-sys](https://gitlab.com/Russel/rust-libdvbv5-sys) crate which
//! provides the Rust FFI to the C API of libdvbv5.

use std::path::Path;

use clap::{App, Arg};

mod channels;

fn main() {
    let matches = App::new("dvb-tune")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Russel Winder <russel@winder.org.uk>")
        .about("Generate a DVBv5 channels file from a DVBv5 transmitter file.")
        .arg(
            Arg::with_name("adapter")
                .short("a")
                .long("adapter")
                .value_name("NUMBER")
                .help("Sets the adapter number to use.")
                .takes_value(true)
                .default_value("0"),
        )
        .arg(
            Arg::with_name("frontend")
                .short("f")
                .long("frontend")
                .value_name("NUMBER")
                .help("Sets the frontend number to use.")
                .takes_value(true)
                .default_value("0"),
        )
        .arg(
            Arg::with_name("output_path")
                .short("o")
                .long("output_path")
                .value_name("PATH")
                .help("Path to output file.")
                .takes_value(true)
                .default_value("dvb-channels.conf"),
        )
        .arg(
            Arg::with_name("timeout_multiplier")
                .short("t")
                .long("timeout_multiplier")
                .value_name("MULTIPLIER")
                .help("Multiplier used for timeouts to obtain tables.")
                .takes_value(true)
                .default_value("1"),
        )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .value_name("verbose")
                .help("Verbosity level: the bigger the integer the more messages get output.")
                .takes_value(true)
                .default_value("0"),
        )
        .arg(
            Arg::with_name("TRANSMITTER_FILE")
                .help("Path to the transmitter file to use as input.")
                .required(true)
                .index(1),
        )
        .get_matches();
    let adapter_number = matches
        .value_of("adapter")
        .unwrap()
        .parse::<u8>()
        .expect("Couldn't parse adapter value as a positive integer.");
    let frontend_number = matches
        .value_of("frontend")
        .unwrap()
        .parse::<u8>()
        .expect("Couldn't parse frontend value as a positive integer.");
    let output_path = Path::new(matches.value_of("output_path").unwrap());
    let timeout_multiplier = matches
        .value_of("timeout_multiplier")
        .unwrap()
        .parse::<u32>()
        .expect("Couldn't parse timeout multiplier value as an unsigned integer.");
    let verbose = matches
        .value_of("verbose")
        .unwrap()
        .parse::<u32>()
        .expect("Couldn't parse verbose value value as an unsigned integer.");
    let transmitter_file_path = Path::new(matches.value_of("TRANSMITTER_FILE").unwrap());
    let frontend_id = dvbv5::FrontendId {
        adapter_number,
        frontend_number,
    };
    match channels::TransmitterData::new(transmitter_file_path) {
        Ok(transmitter_data) => {
            match transmitter_data.scan(
                &frontend_id,
                None,
                Some(timeout_multiplier),
                None,
                None,
                None,
                Some(verbose),
                None,
            ) {
                Ok(channels_data) => {
                    if !channels_data.write(output_path) {
                        println!("**** Error writing channels data ****");
                    }
                }
                Err(_) => println!("**** No receiver, cannot scan. *****"),
            };
        }
        Err(_) => println!("**** Could not get transmitter data. ****"),
    };
}
