



#[derive(PartialEq, Debug)]
pub struct CorrelationEvaluateRow {
    row_name:String,
    x_vals: Vec<f64>,
    y_vals: Vec<f64>
}

impl  CorrelationEvaluateRow {
    fn new(row_name:&str,x_vals:Vec<f64>,y_vals:Vec<f64>) ->  Self{
        Self {
            row_name:String::from(row_name),
            x_vals,
            y_vals
        }
    }
}
pub fn parse_rows_with_names (x_vals:& [f64],y_vals:&[&str])->CorrelationEvaluateRow{

    //initial stage

    //extract names

    
    let string_floats = &y_vals[1..];

    let row_name = match y_vals.get(0){
        Some(row_key) => String::from(*row_key),
        None => String::from("")
    };


    let results = parse_rows(x_vals,&string_floats);

    return CorrelationEvaluateRow {
        row_name,
        x_vals:results.0,
        y_vals:results.1

    };





}
pub fn parse_rows (x_vals:&[f64],y_vals:&[&str]) -> (Vec<f64>,Vec<f64>){
    //optimization ?? memory

    // assumes first item in row is the name of values i.e trait_1 , 12 , 12 , 12,14

    let mut  parsed_x_vals= Vec::new();

    let mut parsed_y_vals = Vec::new();

    for (index,x_val) in x_vals.into_iter().enumerate(){

        if let Some(val) = y_vals.get(index){

            match val.trim().parse::<f64>(){
                Ok(float_type) => {
                    parsed_x_vals.push(*x_val);
                    parsed_y_vals.push(float_type);

                }

                Err(_not_float) => {
                  continue
                }
            }

        }
    }

    (parsed_x_vals,parsed_y_vals)
}

pub fn file_path_validator(){
    todo!()
}
#[cfg(test)]

mod tests{



    use super::*;
    #[test]

    fn test_parsing_rows(){

        //todo add test cases

        
        let x_vals = [12.2,12.1,16.5,11.1];
        let pure_string   = ["12.1","11.1","11.6","11.7"];

        let string_with_nans = ["","1.2","11.1","","4.5","nan"];

        

        assert_eq!(parse_rows(&x_vals,&pure_string),
        (vec![12.2,12.1,16.5,11.1],vec![12.1,11.1,11.6,11.7]));

        assert_eq!(parse_rows(&x_vals, &string_with_nans),(vec![12.1,16.5],vec![1.2,11.1]));
    }


    #[test]
    fn test_extract_keys(){

        //check on allocation


        let x_vals =  [12.2,12.1,16.5,11.1,11.6];


        let expected_results = [
            CorrelationEvaluateRow::new("trait_1",vec![12.1, 16.5, 11.6],vec![1.2,11.1,4.5]),
            CorrelationEvaluateRow::new("trait_2",vec![12.1, 16.5, 11.6],vec![1.6,11.5,4.2]),
            CorrelationEvaluateRow::new("trait_3",vec![12.2, 12.1, 16.5, 11.6],vec![6.5,1.6,11.5,4.2]),
            CorrelationEvaluateRow::new("trait_4", vec![12.2, 12.1, 16.5, 11.6],vec![12.5, 1.6, 11.5, 4.2])
        ];

        for (index,test_case) in [
            ["trait_1","","1.2","11.1","","4.5","nan"],
            ["trait_2","","1.6","11.5","","4.2","nan"],
            ["trait_3","6.5","1.6","11.5","","4.2","12.1"],
            ["trait_4","12.5","1.6","11.5","","4.2","1.1"]
        ].iter().enumerate(){
            
            assert_eq!(parse_rows_with_names(&x_vals, &*test_case),expected_results[index])

        }
    }
}

