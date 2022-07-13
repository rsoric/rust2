extern crate core;
extern crate lazy_static;

use dashmap::DashSet;
use flate2::read::GzDecoder;
use lazy_static::lazy_static;
use protobuf::{Message};
use schema::{Request, Response};
use std::env;
use std::error::Error;
use std::io::Cursor;
use std::io::Read;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

include!(concat!(env!("OUT_DIR"), "/protos/mod.rs"));

lazy_static! {
    static ref WORDS: DashSet<String> = {
        let words = DashSet::new();
        words
    };
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Allow passing an address to listen on as the first argument of this
    // program, but otherwise we'll just set up our TCP listener on
    // 127.0.0.1:8080 for connections.
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "0.0.0.0:3333".to_string());

    // Next up we create a TCP listener which will listen for incoming
    // connections. This TCP listener is bound to the address we determined
    // above and must be associated with an event loop.


    let listener = TcpListener::bind(&addr).await?;
    println!("Listening on: {}", addr);



    loop {
        // Asynchronously wait for an inbound socket.
        let (mut socket, _) = listener.accept().await?;

        //println!("accepted socket");


        tokio::spawn(async move {



            loop{

                let mut len_buffer = [0; 4];

                //println!("loop start");

                if socket.read_exact(&mut len_buffer).await.is_err()
                {
                    //socket.shutdown();
                    return;
                }
                // In a loop, read data from the socket and write the data back.

                let len = u32::from_be_bytes(len_buffer);
                //println!("len: {:?}", len);

                let mut data_buffer = vec![0u8; len as usize];
                //println!("{:?}", data_buffer);

                let _n = socket.read_exact(&mut data_buffer).await.unwrap();

                let in_msg = Request::parse_from_bytes(&data_buffer).unwrap();

                if in_msg.has_postWords() {
                    let words = &in_msg.postWords().data;
                    //println!("{:?}", words);

                    let reader = Cursor::new(words);
                    let mut d = GzDecoder::new(reader);
                    let mut s = String::new();
                    d.read_to_string(&mut s).unwrap();
                    //println!("{}", s);

                    //let wordsToAdd: Vec<&str> = s.split(' ').collect();+
                    /*

                    let wordsToAdd: Vec<&str> = Regex::new("\\s+").unwrap().split(&*s).collect();

                    //println!("{:?}", wordsToAdd);

                    for word in wordsToAdd {
                        //println!("{}", word);
                        WORDS.insert(word.to_owned());
                    }*/
                    //let wordsToAdd: Vec<&str> = re.split(&*s).collect();

                    //let wordsToAdd: Vec<String> = s.split(" ").map(|s| s.to_string()).collect();


                    for word in  s.split_whitespace().map(|s| s.to_string()) {
                        //println!("{}", word);
                        WORDS.insert(word);
                    }


                    let mut bruh = Response::new();
                    bruh.status = ::protobuf::EnumOrUnknown::from_i32(0);

                    let out_bytes: Vec<u8> = bruh.write_to_bytes().unwrap();

                    let len_reply = out_bytes.len().to_be_bytes();

                    socket.write_all(&len_reply).await.expect("failed to write");
                    socket.write_all(&*out_bytes).await.expect("failed to write");
                } else {
                    let mut bruh = Response::new();
                    bruh.status = ::protobuf::EnumOrUnknown::from_i32(0);
                    bruh.counter = WORDS.len() as i32;
                    //println!("sent mf {}", bruh);

                    let out_bytes: Vec<u8> = bruh.write_to_bytes().unwrap();
                    let len_reply = out_bytes.len().to_be_bytes();
                    let len_reply_bytes = [len_reply[4],len_reply[5],len_reply[6],len_reply[7]];


                    //println!("sent mf {:?}",len_reply_bytes );
                    //println!("sent mf {:?}", &*out_bytes);

                    socket
                        .write_all(&len_reply_bytes)
                        .await
                        .expect("failed to write data to socket");
                    socket
                        .write_all(&*out_bytes)
                        .await
                        .expect("failed to write data to socket");

                    WORDS.clear();
                }

            }

        });
    }
}
