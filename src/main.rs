/*
 *  DVBTune — for generating a DVBv5 channels file from a DVBv5 transmitter file.
 *
 *  Copyright © 2019  Russel Winder
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

mod channels;

use clap::{Arg, App};

fn main() {
    let matches = App::new("dvb-tune")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Russel Winder <russel@winder.org.uk>")
        .about("Generate a DVBv5 channels file from a DVBv5 transmitter file.

A transmitter file name.
")
        .arg(Arg::with_name("adapter")
            .short("a")
            .long("adapter")
            .value_name("NUMBER")
            .help("Sets the adapter number to use.")
            .takes_value(true)
            .default_value("0"))
        .arg(Arg::with_name("frontend")
            .short("f")
            .long("frontend")
            .value_name("NUMBER")
            .help("Sets the frontend number to use.")
            .takes_value(true)
            .default_value("0"))
       .arg(Arg::with_name("output_path")
            .short("o")
            .long("output_path")
            .value_name("PATH")
            .help("Path to output file.")
            .takes_value(true)
            .default_value("dvb-channels.conf"))
        .arg(Arg::with_name("TRANSMITTER_FILE")
            .help("Path to the transmitter file to use as input.")
            .required(true)
            .index(1))
        .get_matches();
    let adapter_number = matches.value_of("adapter").unwrap().parse::<u8>().expect("Couldn't parse adapter value as a positive integer.");
    let frontend_number = matches.value_of("frontend").unwrap().parse::<u8>().expect("Couldn't parse frontend value as a positive integer.");
    let output_path = matches.value_of("output_path").unwrap();
    let transmitter_file_path = matches.value_of("TRANSMITTER_FILE").unwrap();
    let frontend_id = dvbv5::types::FrontendId{adapter_number, frontend_number};
    match channels::TransmitterData::new(transmitter_file_path) {
        Ok(transmitter_data) => {
            let channels_data = transmitter_data.scan(&frontend_id);
            channels_data.write(output_path,&frontend_id);
        },
        Err(_) => println!("**** Could not get transmitter data. ****"),
    };
}
