// implementation of extern sorter with rust

//https://betterprogramming.pub/how-to-sort-a-20g-file-in-rust-12abfffbd92b
use std::io::{Write,Read};
use std::fs::File;
 use extsort::*;

//struct corr_results

#[derive(Debug, PartialEq, PartialOrd)]
struct CorrResults {
    row:String,
    rho:f64,
    pval:f64
}

fn ext_sorter (unsorted_vec:Vec<(String,f64,f64)>,file_name:&str,n_top:usize)-> std::io::Result<String>{
    
    File::create(file_name)?;

    let sorter = ExternalSorter::new().with_segment_size(4_000_000);

    let into_iterator = unsorted_vec.into_iter();

    let sorted = sorter.sort_by(into_iterator, |a,b|b.1.abs().partial_cmp(&a.1.abs()).unwrap())?;

    let mut sorted_data: Vec<(String,f64,f64)> = sorted.collect::<Vec<(String,f64,f64)>>().truncate(n_top);
    Ok(sorted_data)

    //corr_results.sort_by(|a,b|b.1.abs().partial_cmp(&a.1.abs()).unwrap());




}