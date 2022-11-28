// implementation of extern sorter with rust

//https://betterprogramming.pub/how-to-sort-a-20g-file-in-rust-12abfffbd92b
use std::fs::File;
use std::cmp::Ordering;
use std::io::{BufWriter, Write};
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



    custom_float_sorter(& mut v);

    let mut buffer = BufWriter::with_capacity(BUFFER_CAPACITY, File::create(&filename).unwrap());
    for (name, rho, p_val, num_overlap) in v.iter() {
        writeln!(buffer, "{},{},{},{}", name, rho, p_val, num_overlap).unwrap();
    }
    buffer.flush().unwrap();

    Ok(String::from("success"))
}

pub fn create_large_file(filename: &str) {
    File::create(filename).unwrap();
}


pub fn custom_float_sorter(v:&mut[(String,f64,f64,i32)]){

//sorter for specific case rust

v.sort_by(|a, b| {
        match (a.1.is_nan()| a.1.is_infinite(), b.1.is_nan()|b.1.is_infinite()) {
            (true, true) => Ordering::Equal,
            (true, false) => Ordering::Greater,
            (false, true) => Ordering::Less,
            (false, false) => b.1.abs().partial_cmp(&a.1.abs()).unwrap(),
        }
    });

}
pub fn float_sorter(unsorted:& mut [f64]){
    //only  for test case
    // custom function to sort floats with nan and inf descending order
    unsorted.sort_by(|&a, &b| {
        match (a.is_nan()| a.is_infinite(), b.is_nan()|b.is_infinite()) {
            (true, true) => Ordering::Equal,
            (true, false) => Ordering::Greater,
            (false, true) => Ordering::Less,
            (false, false) => b.abs().partial_cmp(&a.abs()).unwrap(),
        }
    });
}


mod tests{


    //custom function to to be able to compare  vec with floats
 
    fn eq_with_nan_eq(a: f64, b: f64) -> bool {
        ((a.is_nan()| a.is_infinite()) &&
         (b.is_nan()| b.is_infinite())) || (a == b)
    }
    
    fn vec_compare(va: &[f64], vb: &[f64]) -> bool {
        (va.len() == vb.len()) && 
         va.iter()
           .zip(vb)
           .all(|(a,b)| eq_with_nan_eq(*a,*b))
    }

    #[test]

    fn test_float_sorter(){
        use super::float_sorter;


        
        let  mut test_cases:[(Vec<f64>, Vec<f64>);7] = [( vec![f64::NAN,12.1,11.1,1.1],vec![12.1,11.1,1.1,f64::NAN]),
        (vec![2.1,1.1,2.3,f64::NAN],vec![2.3,2.1,1.1,f64::NAN]),
        (vec![f64::NAN,f64::NAN,9.1],vec![9.1,f64::NAN,f64::NAN]),
        (vec![f64::INFINITY,1.12,f64::INFINITY,42.1],vec![42.1,1.12,f64::INFINITY,f64::INFINITY]),
        (vec![1.1,1.4,1.5,12.1],vec![12.1,1.5,1.4,1.1]),
        (vec![ f64::INFINITY,f64::INFINITY,2.13,5.3,f64::NAN,12.1,f64::INFINITY,5.6,f64::NAN,f64::NAN,8.32],
            vec![12.1, 8.32, 5.6, 5.3, 2.13, f64::INFINITY, f64::INFINITY, f64::NAN, f64::INFINITY, f64::NAN, f64::INFINITY]
        ),
        (vec![1.1,-4.1,4.0,f64::NAN],vec![-4.1,4.0,1.1,f64::NAN])
        ];

        for (test_case,expected_case) in &mut test_cases{

            float_sorter(test_case);
            assert!(vec_compare(test_case, expected_case),"testcase failed for {:?}=={:?}",expected_case,test_case)
        }

    }
    
}