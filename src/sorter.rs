// implementation of extern sorter with rust

//https://betterprogramming.pub/how-to-sort-a-20g-file-in-rust-12abfffbd92b
use std::fs::File;
use std::io::{BufWriter, Read, Write};
//use extsort::*;

const BUFFER_CAPACITY: usize = 4_000_000_000;
//struct corr_results

#[derive(Debug, PartialEq, PartialOrd)]
struct CorrResults(String, f64, f64, i32);

// impl  Sortable for  CorrResults {

//  fn decode<R: Read>(reader: &mut R) -> Option<Self> {

//        todo!()
//    }

//    fn encode<W: Write>(&self, writer: &mut W) {

//        writer.write_fmt(format_args!("{} {} {}",self.0,self.1,self.2)).unwrap();

//    }

// }

// fn ext_sorter (unsorted_vec:Vec<CorrResults>,file_name:&str,n_top:usize)-> std::io::Result<Vec<CorrResults>>{

//    File::create(file_name)?;

//    let sorter = ExternalSorter::new().with_segment_size(4_000_000);

//    let into_iterator = unsorted_vec.into_iter();

//    let sorted = sorter.sort_by(into_iterator, |a,b|b.1.abs().partial_cmp(&a.1.abs()).unwrap())?.collect::<Vec<CorrResults>>();

//sorted.collect::<Vec<CorrResults>>().truncate(n_top);

//    Ok(sorted)

//corr_results.sort_by(|a,b|b.1.abs().partial_cmp(&a.1.abs()).unwrap());

// }

pub fn sort_write_to_file(
    filename: String,
    mut v: Vec<(String, f64, f64, i32)>,
) -> std::io::Result<String> {
    File::create(filename.clone())?;

    v.sort_by(|a, b| b.1.abs().partial_cmp(&a.1.abs()).unwrap());
    let mut buffer = BufWriter::with_capacity(BUFFER_CAPACITY, File::create(&filename).unwrap());
    for (name, rho, p_val, num_overlap) in v.iter() {
        writeln!(buffer, "{},{},{} {}", name, rho, p_val, num_overlap).unwrap();
    }
    buffer.flush().unwrap();

    Ok(String::from("success"))
}

pub fn create_large_file(filename: &str) {
    File::create(filename).unwrap();
}
