extern crate core;
extern crate lazy_static;
extern crate smol;

//use core::num::fmt::Part::Num;

use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::{result, u32, usize};
use std::collections::HashMap;
use std::f32::consts::E;
use std::io::Cursor;
use std::ptr::null;
use protobuf::{EnumOrUnknown, Message};
use schema::{Request, Response};
use flate2::read::GzDecoder;
use dashmap::{DashMap, DashSet};
use protobuf::reflect::RuntimeType::String;
use protobuf::rt::int32_size;
use std::string::String as OtherString;
use std::sync::Arc;
use lazy_static::lazy_static;
use std::mem::transmute;
use smol::{Async, Task};
use smol::io::{AsyncReadExt, AsyncWriteExt};

lazy_static! {
    static ref WORDS: DashSet<OtherString> = {
        let mut words = DashSet::new();
        words
    };
}

include!(concat!(env!("OUT_DIR"), "/protos/mod.rs"));

fn main() {

    smol::block_on(async {
        let listener = Async::<TcpListener>::bind(([0, 0, 0, 0], 3333)).unwrap();

        loop {
            let (stream, _) = listener.accept().await.unwrap();
            smol::spawn(async { handle_connection(stream).await }).detach();
        }
    })
}

async fn handle_connection(mut stream: Async<TcpStream>) {
    // Read the first 1024 bytes of data from the stream
    let mut len_buffer = [0; 4];

    println!("handleconnection");

    let mut check = [0; 4];

    let mut bruh = Response::new();
    bruh.status = ::protobuf::EnumOrUnknown::from_i32(0);

    while(stream.read(&mut len_buffer).await.is_ok())
    {



        let len = u32::from_be_bytes(len_buffer);

        //println!("{:?}", len);

        let mut data_buffer: Box<[u8]> = vec![0; len as usize].into_boxed_slice();
        stream.read(&mut data_buffer);
        println!("{:?}", data_buffer);

        let in_msg = Request::parse_from_bytes(&data_buffer).unwrap();


        if(in_msg.has_postWords())
        {
            let words = &in_msg.postWords().data;
            println!("{:?}", words);

            let mut reader = Cursor::new(words);

            let mut d = GzDecoder::new(reader);
            let mut s = OtherString::new();
            d.read_to_string(&mut s).unwrap();
            println!("{}", s);

            let mut wordsToAdd = s.split(" ");

            for word in wordsToAdd{
                //println!("{}", word);
                WORDS.insert(word.to_owned());
            }

            let out_bytes: Vec<u8> = bruh.write_to_bytes().unwrap();




            let mut len_reply = out_bytes.len().to_be_bytes();

            stream.write_all(&len_reply);
            stream.write_all(&*out_bytes);
            stream.flush().await.unwrap();

        }
        else{
            bruh.counter = WORDS.len() as i32;
            println!("sent mf {}", WORDS.len() as i32);
            let out_bytes: Vec<u8> = bruh.write_to_bytes().unwrap();
            let mut len_reply = out_bytes.len().to_be_bytes();
            stream.write_all(&len_reply);
            stream.write_all(&*out_bytes);
            stream.flush().await.unwrap();
            WORDS.clear();

        }


    }

}
