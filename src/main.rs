use correlation_rust::analysis::Analysis;
use correlation_rust::gn2_parser::LmdbCorrelation;
use std::env;
fn main() {
    use std::time::Instant;
    let now = Instant::now();


//change to cmd argument
    match (env::args().nth(1),env::args().nth(2)) {
        (Some(tmp_file),Some(_corr_all))=>{           
          match LmdbCorrelation::new(&tmp_file).compute_full_dataset(){
            Ok(results) => println!("{:?}",results),
            Err(err)=> println!("{:?}",err)
          }
        }, 
        (Some(tmp_file),None) =>
         match Analysis::compute(&tmp_file) {
            Ok(results) => println!("{:?}", results),
            Err(err) => println!("{:?}", err),
        },

    
    _ => panic!("expected an arugment for the json file"),
    };

    let elapsed = now.elapsed();
    println!("Elapsed:>>>>>>>>>>>>>>>>>>>>{:.2?}", elapsed);
}
