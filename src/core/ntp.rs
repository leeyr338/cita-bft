// Copyright 2019 Cryptape Technologies LLC.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! > To calculate the roundtrip delay d and system clock offset t relative
//! > to the server, the client sets the Transmit Timestamp field in the
//! > request to the time of day according to the client clock in NTP
//! > timestamp format.  For this purpose, the clock need not be
//! > synchronized.  The server copies this field to the Originate
//! > Timestamp in the reply and sets the Receive Timestamp and Transmit
//! > Timestamp fields to the time of day according to the server clock in
//! > NTP timestamp format.
//! >
//! > When the server reply is received, the client determines a
//! > Destination Timestamp variable as the time of arrival according to
//! > its clock in NTP timestamp format.  The following table summarizes
//! > the four timestamps.
//! >
//! >    Timestamp Name          ID   When Generated
//! >    ------------------------------------------------------------
//! >    Originate Timestamp     T1   time request sent by client
//! >    Receive Timestamp       T2   time request received by server
//! >    Transmit Timestamp      T3   time reply sent by server
//! >    Destination Timestamp   T4   time reply received by client
//! >
//! > The roundtrip delay d and system clock offset t are defined as:
//! >
//! > d = (T4 - T1) - (T3 - T2)     t = ((T2 - T1) + (T3 - T4)) / 2.
//!
//! More details at [SNTP](https://tools.ietf.org/html/rfc4330).

use ntp::errors::Error;
use ntp::request;
use time::now_utc;
use time::{Duration, Timespec};

#[derive(Debug, Deserialize, Clone)]
pub struct Ntp {
    pub enabled: bool,
    pub threshold: i64,
    pub address: String,
}

impl Ntp {
    /// Check the system clock offset overflow the threshold
    pub fn is_clock_offset_overflow(&self) -> bool {
        match Ntp::system_clock_offset(self) {
            Ok(offset) => {
                if offset.num_milliseconds().abs() > self.threshold {
                    debug!("System clock seems off by {}", offset);
                    true
                } else {
                    false
                }
            }
            Err(_) => true,
        }
    }

    /// Caclulate the system clock offset relative to the ntp server
    fn system_clock_offset(&self) -> Result<Duration, Error> {
        match request(self.address.clone()) {
            Ok(packet) => {
                let dest = now_utc().to_timespec();
                let orig = Timespec::from(packet.orig_time);
                let recv = Timespec::from(packet.recv_time);
                let transmit = Timespec::from(packet.transmit_time);

                let offset = ((recv - orig) + (transmit - dest)) / 2;

                Ok(offset)
            }
            Err(err) => {
                debug!("Fetch time err: {}", err);
                Err(err)
            }
        }
    }
}
