
/*
Primarily for gn2:
this module is an interface for gn2 and is not necessary
the idea is to read lmdb file directly when computing 
sample correlation remove overhead since most of the files
are huge

lmdb path
 */


use lmdb::{self,Database,Environment,EnvironmentFlags, Transaction};
use serde::de::DeserializeOwned;
use serde::{Serialize,Deserialize};
use std::collections::HashMap;
use std::cmp::Ordering;
use std::path::Path;
use rayon::prelude::*;
use std::f64;
use serde_pickle as pickle;
use std::io::Read;
use crate::correlations::{Pearson,Spearman, Correlation, Compute};
use crate::sorter::sort_write_to_file;
use std::error::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct MyStruct2 {
    data:HashMap<String,Vec<Option<f64>>>,
    creation_date: String,
    strain_names:Vec<String>
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct LmdbCorrelation{
    lmdb_target_path:String,
    lmdb_target_key:String,
    output_file:String,
    primary_trait: Vec<f64>,
    primary_sample_names:Vec<String>,
    file_type:String,
    method:String
}
impl LmdbCorrelation {
    pub fn new(json_file_path:&str) -> Self{
        let mut file = std::fs::File::open(json_file_path).unwrap();
        let mut buff = String::new();
        file.read_to_string(&mut buff).unwrap();

        let results = match serde_json::from_str(&buff) {
            Ok(val) => val,
            Err(error) => panic!("json file not well formatted {:?}", error),
        };
        results
    }
    pub fn unpickle_data<T: DeserializeOwned>(&self,data: &[u8]) -> Result<T, pickle::Error>
where
    T: DeserializeOwned,
{
    serde_pickle::from_slice(data, Default::default())
}


pub fn compute_full_dataset(&self) -> std::io::Result<String> {

    let meta = match self.file_type.to_lowercase().as_str(){
        "lmdb" =>{
             let results = LMDBReader::new(&self.lmdb_target_path).expect("could not read the database")
             .read(self.lmdb_target_key.as_bytes()).expect("Key not found");
             let meta:MyStruct2 = match results{
                Some(data) => self.unpickle_data(&data).expect("failed to unpickle"),
                None => panic!("not data found")
             };     
             meta      
        }
        ("csv" | "txt") =>{
            let meta = LmdbCorrelation::reader(&self, &self.lmdb_target_path);
            meta 
        }
          _ => {panic!("file should either be lmdb or csv/txt file")}
          
    };

    self.compute(meta)


    //do the read operation

}
pub fn compute(&self, data: MyStruct2) -> std::io::Result<String> {
    let set_b: Vec<String> = self.primary_sample_names.iter().cloned().collect();

    let target_indexes: Vec<usize> = data.strain_names
        .iter()
        .enumerate()
        .filter_map(|(i, item_a)| set_b.contains(item_a).then(|| i))
        .collect();

    let mut correlation_results: Vec<(String, f64, f64,i32)> = data.data
        .par_iter()
        .map(|(key, values)| {
            let mut parsed_y_vals: Vec<f64> = Vec::new();
            let mut parsed_x_vals = Vec::new();

            for &index in &target_indexes {
                
                if let (Some(Some(val_y)), Some(val_x)) = (values.get(index), self.primary_trait.get(index)) {
                    parsed_x_vals.push(*val_x);
                    parsed_y_vals.push(*val_y);
                }
            }
            if  parsed_y_vals.len() <= 4 {
                return (key.clone(), f64::NAN, f64::NAN,parsed_x_vals.len() as i32);
            }

            let (rho, p_val) = match self.method.to_lowercase().as_str() {
                "pearson" => Pearson::new(parsed_x_vals.len()).correlate(&parsed_x_vals, &parsed_y_vals),
                _ => Spearman::new(parsed_x_vals.len()).correlate(&parsed_x_vals, &parsed_y_vals),
            };

            (key.clone(), rho, p_val,parsed_x_vals.len() as i32)
        })
        .collect();
    correlation_results.sort_by(|a, b| b.1.abs().partial_cmp(&a.1.abs()).unwrap_or(Ordering::Less));

    if correlation_results.len() > 500 {
        correlation_results.truncate(500);
    }
    sort_write_to_file(self.output_file.clone(),
    correlation_results)
    
}

}

#[derive(Debug)]
pub struct LMDBReader {
    env:Environment,
    db:Database,
}



#[derive(Debug, Serialize, Deserialize)]
struct LMDBDataset {
    creation_date: String,
    sample_names: Vec<String>,
    data:Vec<Vec<String>>
}
impl LMDBReader {
    
    pub fn new(path:&str) -> Result<Self,lmdb::Error>{
        
   let env = Environment::new()
   .set_flags(EnvironmentFlags::NO_READAHEAD | EnvironmentFlags::NO_SUB_DIR)
   .open(Path::new(path))?;   

   let db = env.open_db(None)?;

   Ok(Self { env, db })
 
    }
    pub fn read(&self, key: &[u8]) -> Result<Option<Vec<u8>>, lmdb::Error> {
        let txn = self.env.begin_ro_txn()?;
    
        let result = match txn.get(self.db, &key) {
            Ok(value) => Some(value.to_vec()),
            Err(lmdb::Error::NotFound) => None,
            Err(err) => return Err(err),
        };        

        Ok(result)
    }
}
impl Drop for LMDBReader {
    fn drop(&mut self) {
        self.env.sync(true).expect("Failed to sync LMDB environment");
    }
}


trait CSV_Parser {
    fn reader(&self,file_path:&str)->MyStruct2 ;

    
}
trait Lmdb_Parser {
    fn reader(&self,json_file:&str)->  MyStruct2;
}
fn parse_csv_line(line: &str) -> Result<(String, Vec<Option<f64>>), Box<dyn Error>> {
    let mut values: Vec<Option<f64>> = Vec::new();

    let fields: Vec<&str> = line.split(',').collect();

    if let Some(first_item) = fields.first() {
        let key = first_item.trim().to_string();

        for field in fields.iter().skip(1) {
            let value = field.trim().parse::<f64>().ok();
            values.push(value);
        }

        Ok((key, values))
    } else {
        Err("Failed to parse line".into())
    }
}
impl CSV_Parser for LmdbCorrelation {
    fn reader(&self,json_file:&str) ->MyStruct2{
        let results = Self::new(json_file);

        use std::io::{self, BufRead, BufReader};    
        let file = std::fs::File::open(&results.lmdb_target_path).unwrap(); 
        let mut reader  = BufReader::new(file);

        let  mut col_names = String::new();
        reader.read_line(&mut col_names).expect("failed to parse first line");

        let  col_names:Vec<String> = col_names.split(",")
        .map(|s| s.trim().to_string())
        .skip(1)
        .collect();
    
        let mut data: HashMap<String, Vec<Option<f64>>> = HashMap::new();
        for line in reader.lines() {
            if let Ok(parsable_data) = line {
                let (key, values) = parse_csv_line(&parsable_data).unwrap();
                data.insert(key, values);

            }                
        } 
        MyStruct2 {
            data:data,
            creation_date:String::from("N/A"),

           strain_names:col_names
        }}
    
    }



//csv parser the aim is to both and parse for both

