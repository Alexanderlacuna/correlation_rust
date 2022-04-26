//file reads dataset file and also does the parsing
//aim is to read this as a stream
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;

pub fn file_reader(file_path:&str) ->Result<BufReader<File>, Box<dyn std::error::Error>>{

    let file  = File::open(file_path)?;

    let buf_reader = BufReader::new(file);


    Ok(buf_reader)
}

pub fn add_numbers_2(a:i32,b:i32) -> i32{
    a+b
}




