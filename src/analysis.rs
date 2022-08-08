// entry point
use crate::correlations;
use crate::parser;
pub struct Analysis;
impl Analysis {
    pub fn compute(tmp_file: &str) -> std::io::Result<String> {
        let json_results = parser::JsonData::new(tmp_file);

        //above should return  a results

        let x_vals: Vec<f64> = json_results.x_vals;

        let computation_status = correlations::Compute::new(
            json_results.file_delimiter,
            &json_results.method,
            &json_results.file_path,
            &x_vals,
            &json_results.output_file,
        )
        .compute();

        // let computation = correlations::Compute::filter_top(computation, 500);

        computation_status
    }
}

#[cfg(test)]

mod tests {
    #[test]
    fn test_sum() {
        assert_ne!(1, 5)
    }
}
