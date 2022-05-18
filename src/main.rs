use std::env;
use correlation_rust::parser;
use correlation_rust::correlations;
fn main () {


    let json_file = match env::args().nth(1){
        Some(tmp_file) => {
            let results = parser::JsonData::new(&tmp_file);


            let parsed_x_vals:Vec<f64> = results.x_vals.split(",").map(|f|f.trim().parse::<f64>().unwrap()).collect();
      


            let computation = correlations::Compute::new(
                ',',
                "pearson",
                &results.file_path,
        &parsed_x_vals

            ).compute();

            //println!("{:?}",computation);

        }
        None => panic!("expected an arugment for the json file")
    };





}