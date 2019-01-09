# DVBTune

A program to read a DVB transmitter file, and use a connected DVB tuner to search for all the channels and
create a GStreamer channels file.

This is essentially a Rust version of [dvbv5-scan](https://www.linuxtv.org/wiki/index.php/Dvbv5-scan)
from [libdvbv5](https://linuxtv.org/docs/libdvbv5/). It is an example of using the Rust binding to libdvbv5,
the FFI is [dvbv5_sys](https://crates.io/crates/dvbv5-sys) and the Rust side is [dvbv5](.https://crates.io/crates/dvbv5)
