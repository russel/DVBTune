# DVBTune

GitLab CI: [![GitLab CI](https://gitlab.com/Russel/DVBTune/badges/master/pipeline.svg)](https://gitlab.com/Russel/dvbtune)
&nbsp;&nbsp;&nbsp;&nbsp;
Travis-CI: [![Travis-CI build status](https://travis-ci.org/russel/DVBTune.svg?branch=master)](https://travis-ci.org/russel/DVBTune)

Licence: [![Licence](https://img.shields.io/badge/license-GPL_3-green.svg)](https://www.gnu.org/licenses/gpl-3.0.en.html)

A program to read a DVB transmitter file, and use a connected DVB tuner to search for all the channels and
create a GStreamer channels file.

This is essentially a Rust version of [dvbv5-scan](https://www.linuxtv.org/wiki/index.php/Dvbv5-scan)
from [libdvbv5](https://linuxtv.org/docs/libdvbv5/). It is an example of using the Rust binding to libdvbv5,
the FFI is [dvbv5_sys](https://crates.io/crates/dvbv5-sys) and the Rust side is [dvbv5](.https://crates.io/crates/dvbv5)

## Licence

This code is licenced under GPLv3.
[![Licence](https://www.gnu.org/graphics/gplv3-127x51.png)](https://www.gnu.org/licenses/gpl-3.0.en.html)
