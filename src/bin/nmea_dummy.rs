//! Helper that outputs NMEA $GNTXT messages (with checksum) every second with an increasing counter.
//!
//! # Example messages
//!
//! ```
//! $GNTXT,01,01,02,upcounting timer is at 1*0C
//!
//! $GNTXT,01,01,02,upcounting timer is at 2*0F
//!
//! $GNTXT,01,01,02,upcounting timer is at 3*0E
//! ```
//!
//!

#![deny(missing_docs)]

use std::time::Duration;
use std::thread::sleep;

fn main() {
    let mut counter: u32 = 0;
    loop {
        counter += 1;

        let packet = format!("GNTXT,01,01,02,increasing counter is at {}", counter);

        let mut checksum = 0;
        for b in packet.as_bytes() {
            checksum = checksum ^ b;
        }
        sleep(Duration::new(1, 0));
        print!("${}*{:02X}\r\n", &packet, checksum);

        if counter == u32::max_value() {
            counter = 0;
        }
    }
}
