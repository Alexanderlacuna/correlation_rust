
pub fn parse_rows (x_vals:&[f64],y_vals:&[&str]) -> (Vec<f64>,Vec<f64>){
    //pointer

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
}

