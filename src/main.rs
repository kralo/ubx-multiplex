/*!


NOTE: we use GNTXT for our status messages because they can be easily embedded in the stream
*/


#![allow(unused_variables)]
#![allow(unused_imports)]
#![warn(missing_docs)]


use std::env;
use std::process;
use std::thread;
use std::io::{self, Read, Write, Error};
use std::net::SocketAddr;
use std::net::TcpStream;
use std::net::{TcpListener,ToSocketAddrs};
use std::sync::mpsc::channel;
use std::time::SystemTime;
//use std::comm::SharedChan;

mod helpers;
use helpers::{PassthroughState, LexerState};

const MAX_PACKET_LENGTH: i32 = 1000; //seems arbitrary. inspired by gpsd.h-tail

type Port = u16;

struct Program {
    name: String,
}

impl Program {
    fn new(name: String) -> Program {
        Program { name: name }
    }

    fn usage(&self) {
        println!("usage: {} HOST1 PORT1 HOST2 PORT2", self.name);
    }

    fn print_error(&self, mesg: String) {
        writeln!(io::stderr(), "{}: error: {}", self.name, mesg).unwrap();
    }

    // 	fn print_fail(&self,mesg: String) -> ! {
    // 		self.print_error(mesg);
    // 		self.fail();
    // 	}

    fn exit(&self, status: i32) -> ! {
        process::exit(status);
    }
    fn fail(&self) -> ! {
        self.exit(-1);
    }
}

fn main() {
    // SETUP start
    let mut args = env::args();
    let program = Program::new(args.next().unwrap_or("test".to_string()));

    let host1 = args.next().unwrap_or_else(|| {
        program.usage();
        program.fail();
    });

    let port1 = args.next()
        .unwrap_or_else(|| {
            program.usage();
            program.fail();
        })
        .parse::<Port>()
        .unwrap_or_else(|error| {
            program.print_error(format!("invalid port1 number: {}", error));
            program.usage();
            program.fail();
        });

    let host2 = args.next().unwrap_or_else(|| {
        program.usage();
        program.fail();
    });

    let port2 = args.next()
        .unwrap_or_else(|| {
            program.usage();
            program.fail();
        })
        .parse::<Port>()
        .unwrap_or_else(|error| {
            program.print_error(format!("invalid port2 number: {}", error));
            program.usage();
            program.fail();
        });

/*        let port_mon = args.next()
            .unwrap_or_else(|| {
                program.usage();
                program.fail();
            })
            .parse::<Port>()
            .unwrap_or_else(|error| {
                program.print_error(format!("invalid portmon number: {}", error));
                program.usage();
                program.fail();
            });
*/
    // SETUP end

    /*let listener = TcpListener::bind(&("127.0.0.1", port_mon)).unwrap();


    let (port, chan) : (Port<str>, SharedChan<str>)= SharedChan::new();


        // accept connections and process them, spawning a new thread for each one
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    thread::spawn(move|| {
                        // connection succeeded
                        let chan = chan.clone();
                        handle_monitor(stream, chan);
                    });
                }
                Err(e) => { /* connection failed */ }
            }
        }
*/

    let (tx, rx) = channel::<PassthroughState>();

    let stream1 = TcpStream::connect((host1.as_str(), port1))
        .unwrap_or_else(|error| panic!(error.to_string()));
    let mut input_stream1 = stream1.try_clone().unwrap();


    // while blocked, messages are not retained but simply discarded.
    let handler = thread::spawn(move || {
        let mut block_state = PassthroughState::Unblocked;
        let mut last_notify_second_thread = SystemTime::now();
        loop {
            let mut client_buffer = [0u8; 1024];
            match input_stream1.read(&mut client_buffer) {
                Ok(n) => {
                    // n is number of bytes
                    if n == 0 {
                        panic!("no data from socket1");
                    } else {
                        // always do this when data is available.
                        match rx.try_recv() {
                            Ok(x) => {
                                block_state = x;
                                last_notify_second_thread = SystemTime::now();
                            }
                            // We break only if the channel is closed,
                            // it means that all senders are finished.
                            // Err(e) if e == std::comm::Disconnected => { break; },
                            _ => {}
                        }

                        if block_state == PassthroughState::Blocked {
                            match last_notify_second_thread.elapsed() {
                                Ok(elapsed) => {
                                    if elapsed.as_secs() > 2 {
                                        // force unblock if other sender 'died'
                                        block_state = PassthroughState::Unblocked;
                                        print!("$GNTXT,01,01,02,S1: force-unblock after \
                                                timeout*00\r\n");
                                    }
                                }
                                _ => {}
                            }
                        }
                        if block_state == PassthroughState::Unblocked {
                            // io::stdout().write(b"$GNTXT,01,01,02,S1: \
                            // data follows*79\r\n").unwrap();
                            io::stdout().write(&client_buffer).unwrap();
                            io::stdout().flush().unwrap();
                        }
                    }
                }
                Err(error) => panic!(error.to_string()),
            }
        }});

    let stream2 = TcpStream::connect((host2.as_str(), port2))
        .unwrap_or_else(|error| panic!(error.to_string()));
    let mut input_stream2 = stream2.try_clone().unwrap();

    let handler2 = thread::spawn(move || {
        fn std_handling(x: u8) -> LexerState {
            return if x == 0xb5 {
                LexerState::UbxLeader1
            } else if x == 0x24 {
                // '$'
                LexerState::NmeaDollar
            } else {
                LexerState::GroundState
            };
        }

        let mut last_state = PassthroughState::Unblocked;
        let check_state = |t: PassthroughState, last_state: PassthroughState| {
            if t == PassthroughState::Unblocked && last_state == t {
                // do nothing, do not repeat unblockers
            } else {
                // but always send blockers
                // so other thread can calculate if we lost the connection
                tx.send(t).unwrap();
            }
            return t;
        };


        loop {
            let mut client_buffer = [0u8; 1024];
            match input_stream2.read(&mut client_buffer) {
                Ok(n) => {
                    if n == 0 {
                        panic!("no data from socket2");//program.exit(0);
                    } else {
                        let mut t = LexerState::GroundState;
                        let mut length = 0u16;
                        let mut cur_packet: Vec<u8> = Vec::new();

                        for x in client_buffer[0..n].iter() {
                            match t {
                                LexerState::GroundState => {
                                    cur_packet.clear();
                                    t = std_handling(*x);
                                }
                                LexerState::UbxLeader1 => {
                                    if *x == 0x62 {
                                        t = LexerState::UbxLeader2;
                                    }
                                }
                                LexerState::UbxLeader2 => {
                                    t = LexerState::UbxClassId;
                                }
                                LexerState::UbxClassId => {
                                    t = LexerState::UbxMessageId;
                                }
                                LexerState::UbxMessageId => {
                                    length = *x as u16;
                                    t = LexerState::UbxLength1;
                                }
                                LexerState::UbxLength1 => {
                                    length += (*x as u16) << 8;

                                    t = if (length as i32) <= MAX_PACKET_LENGTH {
                                        LexerState::UbxLength2
                                    } else {
                                        LexerState::GroundState
                                    }
                                }
                                LexerState::UbxLength2 => {
                                    t = LexerState::UbxPayload;
                                }
                                LexerState::UbxPayload => {
                                    length -= 1;
                                    if length == 0 {
                                        t = LexerState::UbxChecksumA;
                                    }// else stay in payload state
                                }
                                LexerState::UbxChecksumA => {
                                    t = LexerState::UbxRecognized;
                                }
                                LexerState::NmeaDollar => {
                                    t = if *x == 0x47 {
                                        // 'G'
                                        LexerState::NmeaPubLead
                                    } else {
                                        LexerState::GroundState
                                    }
                                },
                                LexerState::NmeaPubLead => {
                                    if *x == 0x0D {
                                        // '\r'
                                        t = LexerState::NmeaCr;
                                    } //else stay here
                                },
                                LexerState::NmeaCr => {
                                    t = if *x == 0x0A {// '\n'
                                        LexerState::NmeaRecognized
                                    } else {
                                        LexerState::GroundState
                                    };
                                }
                                // return from the sentences
                                LexerState::NmeaRecognized => {
                                    cur_packet.clear();
                                    t = std_handling(*x);
                                }
                                LexerState::UbxRecognized => {
                                    cur_packet.clear();
                                    t = std_handling(*x);
                                }
                            }

                            cur_packet.push(*x); // append just processed byte to package 'cache'

                            match t {
                                LexerState::UbxRecognized => {
                                    // io::stdout()
                                    // .write(format!("got a finished ublox packet\n").as_bytes())
                                    // .unwrap();
                                    //io::stdout().write(&cur_packet).unwrap();
                                    // inspect ublox packet
                                    /*io::stdout()
                                        .write(format!("2>got ublox packet cls:0x{:02X} \
                                                        msg:0x{:02X} len:{}\n",
                                                       cur_packet[2],
                                                       cur_packet[3],
                                                       cur_packet.len())
                                            .as_bytes())
                                        .unwrap();
*/
                                    if cur_packet[2] == 0x01 && cur_packet[3] == 0x06 {
                                        // NAV-SOL
                                        if cur_packet[17] & 0x02 == 0x02 {
                                            if last_state == PassthroughState::Unblocked {
                                                print!("$GNTXT,01,01,02,S2: acquiring diff \
                                                fix*2F\r\n");
                                            }
                                            last_state = check_state(PassthroughState::Blocked,
                                                                     last_state);
                                        } else {
                                            if last_state == PassthroughState::Blocked {
                                                print!("$GNTXT,01,01,02,S2: \
                                                loosing diff fix*00\r\n");
                                            }

                                            last_state = check_state(PassthroughState::Unblocked,
                                                                     last_state);
                                        }
                                    }
                                },
                                _ => {}
                            }
                        }
                        // was mit dem Rest ?
                        /*io::stdout().write(format!("read {:?} bytes [", n).as_bytes()).unwrap();
                        for x in client_buffer[0..n].iter() {
                            io::stdout().write(format!("{:02X} ", *x).as_bytes()).unwrap();
                        }
                        io::stdout().write(format!("]\n").as_bytes()).unwrap();*/
                        // io::stdout().write(format!("{:02X}", x).as_bytes()).unwrap();
                        //io::stdout().write(b"$GNTXT,01,01,02,S2: data follows*7A\r\n").unwrap();
                        if last_state == PassthroughState::Blocked {
                            // only write content if unblocked. Else the first stream is better.
                            io::stdout().write(&client_buffer).unwrap();
                        }
                        io::stdout().flush().unwrap();
                    }

                }
                Err(error) => panic!(error.to_string()),
            }
        }
    });

    // let output_stream = &mut stream;
    let mut user_buffer = String::new();

    loop {
        // let him do something so this loop doesnt eat 100% CPU
        io::stdin().read_line(&mut user_buffer).unwrap();

        // output_stream.write(user_buffer.as_bytes()).unwrap();
        // output_stream.flush().unwrap();
    }
}
