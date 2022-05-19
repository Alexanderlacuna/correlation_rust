use std::env;
use correlation_rust::analysis::Analysis;
fn main () {
    match env::args().nth(1){
        Some(tmp_file) => {
         
            match Analysis::compute(&tmp_file){
                Ok(results) => println!("{:?}",results),
                Err(err) => println!("{:?}",err)
            }
        }
        None => panic!("expected an arugment for the json file")
    };

}
