use core::panic;
//file reads dataset file and also does the parsing
//aim is to read this as a stream
use std::fs::File;
use std::io::{BufReader, BufRead};
use std::path::Path;


pub struct BufferReader {
    reader:BufReader<File>
}


impl BufferReader{
    pub fn new(file_path:&str) -> std::io::Result<Self>{
        let file = File::open(Path::new(file_path))?;
        //add file validator

        return Ok(Self{
            reader:BufReader::new(file)
        });

        

    }

    pub fn read_line <'a>(&mut self,buf:&'a mut String) -> Option<std::io::Result<&'a mut String>>{
        //to avoid allocation for each new line

        buf.clear();

        self.reader.read_line(buf).map(|u  | {
            if  u == 0  as usize {
                return None
            }

            Some(buf)

        
        }).transpose()
}

    pub fn fetch_keys <'a>(&self,row:& 'a str) -> (&str,[&str]){


        // parse the line and return (trait_name and trait_values)

        //parse the other  body to float type

        let mut buf = String::new();
       // should be taken to  own  line


        let sample_buffer =  self.read_line(&mut buf);

        let sample_names = match sample_buffer{

            Some(val)=> {
                if let Ok(new_vak) = val{

                    new_vak

                }

                else {
                    panic!("error happened");
                }
            },

            None =>  panic!("no sample values found")

        };

 
    

        let string_vec : Vec<&str>= row.split(",").collect();

        //issue is that spaces are included

        return [string_vec[0],string_vec[1..]];
    }  
pub fn row_parser <'a>(&self,row :&'a mut String) ->(& 'a str,Vec<&'a str>){

    let parsed_row:Vec<&str> = row.split(",").map(|s|s.trim()).collect();

    let (trait_name,trait_vals) =  (parsed_row[0],parsed_row[1..].to_vec());

    (parsed_row[0],parsed_row[1..].to_vec())

}     
}


pub fn file_reader(file_path:&str) ->Result<BufReader<File>, Box<dyn std::error::Error>>{

    let file  = File::open(Path::new(file_path))?;

    let buf_reader = BufReader::new(file);
    Ok(buf_reader)
}



pub fn add_numbers_2(a:i32,b:i32) -> i32{
    a+b
}


#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_file_reader_error(){
        assert!(file_reader("./non_file.txt").is_err());
    }


    #[test]

    fn test_file_reader() {
      let expected_results = file_reader("/home/kabui/correlation_rust/src/mock_dataset.txt");

      assert!(expected_results.is_ok());

    }

    # [test]
    fn test_file_reader_2(){
    
        let mut buffer_reader = BufferReader::new("/home/kabui/correlation_rust/src/mock_dataset.txt").unwrap();
        let mut buf = String::new();
        let  mut data = [
            "9. ,5. ,0. ,7. ,6. ,1. ,5. ,0.\n",
            "3. ,9. ,4. ,2. ,2. ,7. ,2. ,4.\n",
             "7. ,1. ,7. ,2. ,2. ,5. ,6. ,5.\n",
            "4. ,5. ,1. ,1. ,6. ,1. ,8. ,7.\n"
            ].iter();


        while let Some(val) = buffer_reader.read_line(& mut buf){

            if let Some(expected) = data.next(){
                assert_eq!(*expected,val.unwrap())
            }

        }

    }


}
