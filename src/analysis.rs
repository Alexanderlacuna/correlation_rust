// entry point
use crate::correlations;
use crate::parser;

pub struct Analysis;

impl Analysis {
    /// Compute correlation from JSON configuration file.
    /// 
    /// Uses `file_type` field to determine mode:
    /// - "csv" (default): Standard CSV mode
    /// - "lmdb": LMDB mode with strain-based column extraction
    pub fn compute(tmp_file: &str) -> std::io::Result<String> {
        let json_results = parser::JsonData::new(tmp_file);

        let x_vals: Vec<f64> = json_results.x_vals;

        // Determine mode based on file_type
        let is_lmdb_mode = json_results.file_type == "lmdb";

        let computation_status = if is_lmdb_mode {
            // LMDB mode - requires strains for column extraction
            let strains_ref = json_results.strains.as_deref();
            correlations::Compute::new_with_strains_parallel(
                json_results.file_delimiter,
                &json_results.method,
                &json_results.file_path,
                &x_vals,
                &json_results.output_file,
                strains_ref,
                json_results.parallel,
            )
            .compute()
        } else {
            // CSV mode (default)
            correlations::Compute::new_with_strains_parallel(
                json_results.file_delimiter,
                &json_results.method,
                &json_results.file_path,
                &x_vals,
                &json_results.output_file,
                None,
                json_results.parallel,
            )
            .compute()
        };

        computation_status
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_sum() {
        assert_ne!(1, 5)
    }

    /// Test that Analysis::compute correctly handles strains from JSON
    #[test]
    fn test_analysis_with_strains() {
        // Create a temporary JSON file with strains
        let tmp_dir = std::env::temp_dir();
        let json_path = tmp_dir.join("test_with_strains.json");

        {
            let mut file = std::fs::File::create(&json_path).unwrap();
            file.write_all(
                br#"{
                "file_path": "/tmp/test_lmdb/",
                "x_vals": [1.0, 2.0, 3.0],
                "strains": ["BXD1", "BXD2", "BXD5"],
                "file_type": "lmdb",
                "sample_values": "test",
                "method": "pearson",
                "file_delimiter": ",",
                "output_file": "/tmp/test_output.txt",
                "parallel": false
            }"#
            )
            .unwrap();
        }

        // Verify JSON parsing works with strains
        let json_data = parser::JsonData::new(json_path.to_str().unwrap());
        assert!(json_data.strains.is_some());
        assert_eq!(json_data.strains.as_ref().unwrap().len(), 3);
        assert_eq!(json_data.strains.as_ref().unwrap()[0], "BXD1");
        assert_eq!(json_data.parallel, false);

        // Clean up
        std::fs::remove_file(json_path).ok();
    }

    /// Test that Analysis::compute handles JSON without strains (backward compat)
    #[test]
    fn test_analysis_without_strains() {
        let tmp_dir = std::env::temp_dir();
        let json_path = tmp_dir.join("test_without_strains.json");

        {
            let mut file = std::fs::File::create(&json_path).unwrap();
            file.write_all(
                br#"{
                "file_path": "tests/data/db300.txt",
                "x_vals": [1.0, 2.0, 3.0],
                "sample_values": "test",
                "method": "pearson",
                "file_delimiter": ",",
                "output_file": "/tmp/test_output.txt"
            }"#
            )
            .unwrap();
        }

        let json_data = parser::JsonData::new(json_path.to_str().unwrap());
        assert!(json_data.strains.is_none());

        std::fs::remove_file(json_path).ok();
    }
}
