use std::env;
use correlation_rust::analysis::Analysis;
fn main () {

    use std::time::Instant;
    let now = Instant::now();

    match env::args().nth(1){
        Some(tmp_file) => {
         
            match Analysis::compute(&tmp_file){
                Ok(results) => println!("{:?}",results),
                Err(err) => println!("{:?}",err)
            }
        }
        None => panic!("expected an arugment for the json file")
    };


    let elapsed = now.elapsed();
    println!("Elapsed:>>>>>>>>>>>>>>>>>>>>{:.2?}", elapsed);

}
