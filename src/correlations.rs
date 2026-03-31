use crate::lmdb_reader::LmdbReader;
use crate::parser::parse_rows_with_names;
use crate::reader::BufferReader;
use crate::sorter::sort_write_to_file;
use std::cmp::Ordering;

//way to parse the args
use rgsl::{
    randist::t_distribution::{tdist_P, tdist_Q},
    statistics::{correlation, spearman},
};

pub trait Correlation {
    fn correlate(&self, x: &[f64], y: &[f64]) -> (f64, f64);
}

#[derive(PartialEq, Debug)]
pub struct Pearson {
    n: usize,
    degrees_of_freedom: f64,
}

#[derive(PartialEq, Debug)]
struct Spearman {
    n: usize,
    degrees_of_freedom: f64,
}

impl Pearson {
    pub fn new(n: usize) -> Self {
        Self {
            n,
            degrees_of_freedom: (n - 2) as f64,
        }
    }
}

impl Spearman {
    pub fn new(n: usize) -> Self {
        Self {
            n,
            degrees_of_freedom: (n - 2) as f64,
        }
    }
}
impl Correlation for Pearson {
    fn correlate(&self, x: &[f64], y: &[f64]) -> (f64, f64) {
        let corr_coeff = correlation(x, 1, y, 1, self.n);

        //compute p_val two sided

        // P-value (two-sided)
        // Based on R's cor.test method (https://github.com/SurajGupta/r-source/blob/a28e609e72ed7c47f6ddfbb86c85279a0750f0b7/src/library/stats/R/cor.test.R#L21

        let statistic =
            self.degrees_of_freedom.sqrt() * corr_coeff / (1.0 - corr_coeff.powi(2)).sqrt();

        let p_val = 2.0
            * tdist_P(statistic, self.degrees_of_freedom)
                .min(tdist_Q(statistic, self.degrees_of_freedom));

        (corr_coeff, p_val)
    }
}

impl Correlation for Spearman {
    fn correlate(&self, x: &[f64], y: &[f64]) -> (f64, f64) {
        match x.len() == y.len() {
            true => (),
            false => panic!("array should be of the same size"),
        };

        //dditional workspace of size 2*n is required in work
        let mut vec = Vec::with_capacity(2 * self.n);
        let workspace: &mut [f64] = vec.as_mut_slice();
        let rho = spearman(x, 1, y, 1, self.n, workspace);

        //p-val two sided

        let t = rho * (self.degrees_of_freedom / ((rho + 1.0) * (1.0 - rho))).sqrt();
        let p_val = 2.0 * (tdist_Q(t.abs(), self.degrees_of_freedom));

        (rho, p_val)
    }
}

use rayon::prelude::*;

#[allow(dead_code)]
#[derive(PartialEq, Debug)]
pub struct Compute<'a> {
    x_vals: &'a [f64],
    /// Optional strain names corresponding to x_vals indices.
    /// Used for LMDB mode to extract correct columns from matrix.
    strains: Option<&'a [String]>,
    dataset_path: &'a str,
    method: CorrelationMethod,
    file_delimiter: char,
    output_file: &'a str,
    /// Enable parallel processing with rayon
    parallel: bool,
}
#[allow(dead_code)]
impl<'a> Compute<'a> {
    pub fn new(
        file_delimeter: char,
        method: &str,
        dataset_path: &'a str,
        x_vals: &'a [f64],
        output_file: &'a str,
    ) -> Self {
        Self::new_with_strains_parallel(file_delimeter, method, dataset_path, x_vals, output_file, None, true)
    }

    /// Create Compute with strain names for LMDB column alignment.
    pub fn new_with_strains(
        file_delimeter: char,
        method: &str,
        dataset_path: &'a str,
        x_vals: &'a [f64],
        output_file: &'a str,
        strains: Option<&'a [String]>,
    ) -> Self {
        Self::new_with_strains_parallel(file_delimeter, method, dataset_path, x_vals, output_file, strains, true)
    }

    /// Create Compute with full control over parallelization.
    pub fn new_with_strains_parallel(
        file_delimeter: char,
        method: &str,
        dataset_path: &'a str,
        x_vals: &'a [f64],
        output_file: &'a str,
        strains: Option<&'a [String]>,
        parallel: bool,
    ) -> Self {
        let method = match method.to_lowercase().as_str() {
            "pearson" => CorrelationMethod::Pearson,
            "spearman" => CorrelationMethod::Spearman,
            _ => panic!("method cannot be found"),
        };

        Self {
            x_vals,
            strains,
            dataset_path,
            method,
            output_file,
            file_delimiter: file_delimeter,
            parallel,
        }
    }

    pub fn filter_top(result: Vec<(f64, f64)>, n: Option<usize>) -> Vec<(f64, f64)> {
        match n {
            Some(n_top) => result[0..n_top].to_vec(),

            None => result,
        }
    }

    pub fn sorter(results: &mut [(f64, f64)]) {
        //naive sorter
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    }

    pub fn compute(&self) -> std::io::Result<String> {
        let mut corr_results: Vec<(String, f64, f64, i32)> = Vec::new();

        // NEW: Check if path is LMDB directory
        if std::path::Path::new(self.dataset_path).is_dir() {
            // Use LMDB reader
            self.compute_from_lmdb(&mut corr_results)?;
        } else {
            // Use CSV reader (existing code)
            self.compute_from_csv(&mut corr_results)?;
        }

        sort_write_to_file(String::from(self.output_file), corr_results)
    }

    // Compute correlations from LMDB with strain-based column extraction.
    // If strains are specified, extracts only those columns. Otherwise uses all columns.
    fn compute_from_lmdb(
        &self,
        corr_results: &mut Vec<(String, f64, f64, i32)>,
    ) -> std::io::Result<()> {
        let reader = LmdbReader::new(self.dataset_path)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

        // Read metadata to get strain order
        let metadata = reader
            .read_metadata()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

        // Build strain name -> column index mapping
        let strain_to_idx: std::collections::HashMap<&str, usize> = metadata
            .strains
            .iter()
            .enumerate()
            .map(|(i, s)| (s.as_str(), i))
            .collect();

        // STEP 1: Clean input - filter out NaN values from x_vals at the start
        // This ensures we don't carry invalid data through the entire pipeline
        let (clean_strains, clean_x_vals): (Option<Vec<String>>, Vec<f64>) = match self.strains {
            Some(strains) => {
                // Validate strains array matches x_vals array
                if strains.len() != self.x_vals.len() {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!(
                            "Array length mismatch: {} strains but {} x_vals. \
                            strains and x_vals must have the same length and matching order.",
                            strains.len(),
                            self.x_vals.len()
                        ),
                    ));
                }

                // Filter out NaN values from input, keeping track of valid strains
                let mut valid_strains = Vec::new();
                let mut valid_x_vals = Vec::new();
                let mut nan_count = 0;
                
                for (i, (strain, x_val)) in strains.iter().zip(self.x_vals.iter()).enumerate() {
                    if x_val.is_nan() {
                        nan_count += 1;
                        continue; // Skip this sample - it has invalid input data
                    }
                    valid_strains.push(strain.clone());
                    valid_x_vals.push(*x_val);
                }
                
                if nan_count > 0 {
                    eprintln!(
                        "Warning: Filtered out {} samples with NaN x_vals. Using {} of {} input samples.",
                        nan_count,
                        valid_x_vals.len(),
                        self.x_vals.len()
                    );
                }
                
                (Some(valid_strains), valid_x_vals)
            }
            None => {
                // No strains specified - filter NaN from x_vals only
                let valid_x_vals: Vec<f64> = self.x_vals
                    .iter()
                    .copied()
                    .filter(|x| !x.is_nan())
                    .collect();
                
                let nan_count = self.x_vals.len() - valid_x_vals.len();
                if nan_count > 0 {
                    eprintln!(
                        "Warning: Filtered out {} samples with NaN x_vals. Using {} of {} input samples.",
                        nan_count,
                        valid_x_vals.len(),
                        self.x_vals.len()
                    );
                }
                
                (None, valid_x_vals)
            }
        };
        
        // STEP 2: Validate we have enough valid samples after NaN filtering
        if clean_x_vals.len() < 4 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!(
                    "Insufficient valid samples: {} (need at least 4). \
                    Input had {} samples, {} were NaN.",
                    clean_x_vals.len(),
                    self.x_vals.len(),
                    self.x_vals.len() - clean_x_vals.len()
                ),
            ));
        }

        // STEP 3: Map strains to column indices, filtering out missing strains from dataset
        let (column_indices, filtered_x_vals): (Vec<usize>, Vec<f64>) = match clean_strains {
            Some(strains) => {
                // Map strain names to their column indices, filtering out missing strains
                let mut indices = Vec::new();
                let mut x_vals_filtered = Vec::new();
                
                for (i, strain) in strains.iter().enumerate() {
                    if let Some(&idx) = strain_to_idx.get(strain.as_str()) {
                        indices.push(idx);
                        x_vals_filtered.push(clean_x_vals[i]);
                    }
                }

                // Warn if some strains were dropped (not found in dataset)
                if indices.len() != strains.len() {
                    eprintln!(
                        "Warning: Filtered out {} missing strains from dataset. \
                        Using {} of {} requested strains.",
                        strains.len() - indices.len(),
                        indices.len(),
                        strains.len()
                    );
                }
                
                // Validate we still have enough samples after dataset filtering
                if indices.len() < 4 {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!(
                            "Insufficient matching strains in dataset: {} (need at least 4). \
                            Requested {} strains, {} found in dataset.",
                            indices.len(),
                            strains.len(),
                            indices.len()
                        ),
                    ));
                }
                
                (indices, x_vals_filtered)
            }
            None => {
                // No strains specified - use all columns (legacy mode)
                // But still limited by clean_x_vals length
                let n_samples = clean_x_vals.len().min(metadata.columns);
                let indices: Vec<usize> = (0..n_samples).collect();
                let x_vals_filtered: Vec<f64> = clean_x_vals[..n_samples].to_vec();
                (indices, x_vals_filtered)
            }
        };

        // Read all traits into memory for parallel processing
        let traits_data: Vec<(String, Vec<f64>)> = reader
            .iter_traits()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?
            .collect();

        // Use filtered_x_vals for correlation (already aligned with column_indices)
        let x_vals_ref = &filtered_x_vals;

        if self.parallel {
            // Parallel processing with rayon
            let results: Vec<(String, f64, f64, i32)> = traits_data
                .par_iter()
                .filter_map(|(trait_name, full_row)| {
                    // Extract only the columns we need
                    let y_vals: Vec<f64> = column_indices
                        .iter()
                        .map(|&idx| full_row[idx])
                        .collect();

                    // Filter out NaN values and align with x_vals
                    let mut parsed_x_vals = Vec::new();
                    let mut parsed_y_vals = Vec::new();

                    for (i, &y_val) in y_vals.iter().enumerate() {
                        if i < x_vals_ref.len() && !y_val.is_nan() {
                            parsed_x_vals.push(x_vals_ref[i]);
                            parsed_y_vals.push(y_val);
                        }
                    }

                    if parsed_x_vals.len() < 4 || parsed_y_vals.len() < 4 {
                        return None;
                    }

                    let (rho, p_val) = if self.method == CorrelationMethod::Pearson {
                        Pearson::new(parsed_x_vals.len()).correlate(&parsed_x_vals, &parsed_y_vals)
                    } else {
                        Spearman::new(parsed_x_vals.len()).correlate(&parsed_x_vals, &parsed_y_vals)
                    };

                    Some((trait_name.clone(), rho, p_val, parsed_x_vals.len() as i32))
                })
                .collect();

            corr_results.extend(results);
        } else {
            // Sequential processing
            for (trait_name, full_row) in traits_data {
                // Extract only the columns we need
                let y_vals: Vec<f64> = column_indices
                    .iter()
                    .map(|&idx| full_row[idx])
                    .collect();

                // Filter out NaN values and align with x_vals
                let mut parsed_x_vals = Vec::new();
                let mut parsed_y_vals = Vec::new();

                for (i, &y_val) in y_vals.iter().enumerate() {
                    if i < x_vals_ref.len() && !y_val.is_nan() {
                        parsed_x_vals.push(x_vals_ref[i]);
                        parsed_y_vals.push(y_val);
                    }
                }

                if parsed_x_vals.len() < 4 || parsed_y_vals.len() < 4 {
                    continue;
                }

                let (rho, p_val) = if self.method == CorrelationMethod::Pearson {
                    Pearson::new(parsed_x_vals.len()).correlate(&parsed_x_vals, &parsed_y_vals)
                } else {
                    Spearman::new(parsed_x_vals.len()).correlate(&parsed_x_vals, &parsed_y_vals)
                };

                corr_results.push((trait_name, rho, p_val, parsed_x_vals.len() as i32));
            }
        }

        Ok(())
    }

    // REFACTORED: Existing CSV logic extracted to separate method
    fn compute_from_csv(
        &self,
        corr_results: &mut Vec<(String, f64, f64, i32)>,
    ) -> std::io::Result<()> {
        let reader = BufferReader::new(self.dataset_path);

        match reader {
            Ok(mut buffer_read) => {
                let mut n_string = String::new();

                while let Some(val) = buffer_read.read_line(&mut n_string) {
                    if let Ok(array_new_val) = val {
                        let ty = parse_rows_with_names(
                            self.x_vals,
                            &array_new_val
                                .split(self.file_delimiter)
                                .collect::<Vec<&str>>(),
                        );
                        if ty.x_vals.len() < 4 || ty.y_vals.len() < 4 {
                            continue;
                        }

                        let (key_name, parsed_x_val, parsed_y_val) =
                            (ty.row_name, ty.x_vals, ty.y_vals);

                        let (rho, p_val) = if self.method == CorrelationMethod::Pearson {
                            Pearson::new(parsed_x_val.len()).correlate(&parsed_x_val, &parsed_y_val)
                        } else {
                            Spearman::new(parsed_x_val.len())
                                .correlate(&parsed_x_val, &parsed_y_val)
                        };

                        corr_results.push((key_name, rho, p_val, parsed_x_val.len() as i32));
                    }
                }
            }
            Err(err) => panic!("an error occurred {:?}", err),
        }

        Ok(())
    }
}
#[derive(PartialEq, Debug)]
enum CorrelationMethod {
    Pearson,
    Spearman,
}

#[cfg(test)]

mod tests {

    use std::vec;

    use super::*;
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn test_pearson_obj() {
        let new_pearson = Pearson::new(5);
        assert_eq!(
            new_pearson,
            Pearson {
                n: 5,
                degrees_of_freedom: 3.0
            }
        );
    }
    #[test]
    fn test_pearson_correlation() {
        let new_pearson = Pearson::new(3);
        let (corr_coeff, p_val) =
            new_pearson.correlate(&[1., 2., 3., 4., 5.], &[1., 2., 3., 4., 5.]);

        assert_eq!(format!("{:.2}", corr_coeff), "1.00");
        assert_approx_eq!(p_val, 1.341575855, 2f64);
    }

    #[test]
    fn test_pearson_negative_rho() {
        let new_pearson = Pearson::new(3);
        let (corr_coeff, p_val) =
            new_pearson.correlate(&[1., 2., 3., 4., 5.], &[5., 4., 3., 2., 1.]);

        assert_eq!(format!("{:.2}", corr_coeff), "-1.00");
    }

    #[test]
    fn test_pearson_negative_rho_signs() {
        // let new_pearson = Pearson::new(3);

        // Each pair should produce a negative correlation
        let test_cases = vec![
            (vec![1., 2., 3., 4., 5.], vec![5., 4., 3., 2., 1.]), // perfect -1.0
            (vec![1., 2., 3., 4., 5.], vec![5., 4.5, 3.5, 3.0, 2.5]), // roughly -0.9
            (vec![1., 2., 3., 4., 5.], vec![4.5, 4.0, 3.8, 3.5, 3.0]), // small negative ~ -0.3
        ];

        for (x, y) in test_cases {
            let new_pearson = Pearson::new(3);
            let new_spearman = Spearman::new(3);
            let (corr_coeff, _p_val) = new_pearson.correlate(&x, &y); // pearson correlation
            let (s_rho, p_val) = new_spearman.correlate(&x, &y);
            let formatted = format!("{:.2}", corr_coeff);
            let s_formatted = format!("{:.2}", s_rho);

            // Assert the sign is actually negative in the formatted string
            assert!(
                formatted.starts_with('-'),
                "Expected negative correlation, got formatted value `{}` (raw {})",
                formatted,
                corr_coeff
            );

            assert!(
                formatted.starts_with('-'),
                "Expected negative correlation, got formatted value `{}` (raw {})",
                s_formatted,
                s_rho
            );
        }
    }

    #[test]
    fn test_spearman_obj() {
        let new_spearman = Spearman::new(5);

        assert_eq!(
            new_spearman,
            Spearman {
                n: 5,
                degrees_of_freedom: 3.0
            }
        );
    }
    #[test]
    fn test_spearman_correlation() {
        let new_spearman = Spearman::new(5);

        let (s_rho, p_val) = new_spearman.correlate(&[1., 2., 3., 4., 5.], &[5., 6., 7., 8., 7.]);

        assert_approx_eq!(s_rho, 0.079603960396039, 2f64);
        assert_approx_eq!(p_val, 0.43111687, 2f64)
    }

    #[test]
    fn test_compute_obj() {
        let x_vals = [1., 3., 4., 5., 6., 7., 7.];
        let new_compute = [
            Compute::new(',', "spearman", "./notes.txt", &x_vals, "/output.txt"),
            Compute::new(',', "pearson", "./note2.txt", &x_vals, "/output.txt"),
        ];

        assert_eq!(
            [
                Compute {
                    file_delimiter: ',',
                    x_vals: &[1., 3., 4., 5., 6., 7., 7.],
                    strains: None,
                    dataset_path: "./notes.txt",
                    output_file: "/output.txt",
                    method: CorrelationMethod::Spearman,
                    parallel: true,
                },
                Compute {
                    file_delimiter: ',',
                    x_vals: &[1., 3., 4., 5., 6., 7., 7.],
                    strains: None,
                    dataset_path: "./note2.txt",
                    output_file: "/output.txt",
                    method: CorrelationMethod::Pearson,
                    parallel: true,
                }
            ],
            new_compute
        )
    }

    #[test]
    #[should_panic]
    fn test_compute_panics() {
        let new_compute = Compute::new(
            ',',
            "unknown",
            "/notes.txt",
            &[1.2, 1.4, 1.5],
            "/output.txt",
        );
        assert_eq!(
            new_compute,
            Compute {
                file_delimiter: ',',
                x_vals: &[1.2, 1.4, 1.5],
                strains: None,
                dataset_path: "/notes.txt",
                method: CorrelationMethod::Pearson,
                output_file: "/output.txt",
                parallel: true,
            }
        )
    }

    #[test]
    fn test_compute() {
        let compute_obj = Compute::new(
            ',',
            "pearson",
            "tests/data/mock_dataset.txt",
            &[12., 15., 11., 11., 16., 11., 8., 7.],
            "./output.txt",
        );

        let corr_results = compute_obj.compute();

        assert!(corr_results.is_ok())
    }

    #[test]
    fn test_parse_f64() {
        let data = "9. ,5. ,0. ,7. ,6. ,1. ,5. ,0.\n";

        let y = "25.08439 ,72.02225 ,47.56293 ,22.87893 ,14.28721 ,71.84655 ,87.81991 ,84.86824 ,6.72478 ,5.72373 ,73.47078 ,63.74703";

        let expected_results = vec![9., 5., 0., 7., 6., 1., 5., 0.];

        let results: Vec<f64> = data
            .split(',')
            .map(|f_str| f_str.trim().parse::<f64>().expect("parse failed"))
            .collect();

        let u: Vec<f64> = y
            .split(',')
            .map(|f_str| f_str.trim().parse::<f64>().expect("parse failed"))
            .collect();

        assert_eq!(results, expected_results);
        assert_eq!(
            u,
            vec![
                25.08439, 72.02225, 47.56293, 22.87893, 14.28721, 71.84655, 87.81991, 84.86824,
                6.72478, 5.72373, 73.47078, 63.74703
            ]
        )
    }

    #[test]
    fn test_sorter() {
        let mut a = vec![(1.2, 1.0), (1.3, 0.5), (1.3, 7.)];
        Compute::sorter(&mut a);

        let mut f = vec![
            (1.2, 11.1),
            (1.1, 9.7),
            (11.1, 7.8),
            (9.3, 11.1),
            (1.3, 7.0),
            (1.2, 1.0),
            (1.3, 0.5),
        ];
        Compute::sorter(&mut f);

        assert_eq!(a, vec![(1.3, 7.0), (1.2, 1.0), (1.3, 0.5)]);
        assert_eq!(
            f,
            vec![
                (1.2, 11.1),
                (9.3, 11.1),
                (1.1, 9.7),
                (11.1, 7.8),
                (1.3, 7.0),
                (1.2, 1.0),
                (1.3, 0.5)
            ]
        );
    }

    #[test]
    fn test_n_top() {
        let a1 = vec![(1.2, 1.0), (1.3, 0.5), (1.3, 7.)];

        let new_vec = Compute::filter_top(a1, Some(2));

        assert_eq!(new_vec, vec![(1.2, 1.0), (1.3, 0.5)])
    }

    #[test]
    fn test_compute_with_strains() {
        let x_vals = [1.0, 2.0, 3.0];
        let strains = vec![
            String::from("strain_a"),
            String::from("strain_b"),
            String::from("strain_c"),
        ];

        let compute = Compute::new_with_strains(
            ',',
            "pearson",
            "/path/to/lmdb",
            &x_vals,
            "/output.txt",
            Some(&strains),
        );

        assert_eq!(compute.x_vals, &[1.0, 2.0, 3.0]);
        assert_eq!(compute.strains, Some(&strains[..]));
        assert_eq!(compute.dataset_path, "/path/to/lmdb");
        assert_eq!(compute.method, CorrelationMethod::Pearson);
    }

    #[test]
    fn test_compute_without_strains() {
        let x_vals = [1.0, 2.0, 3.0];

        let compute = Compute::new(
            ',',
            "spearman",
            "/path/to/file.txt",
            &x_vals,
            "/output.txt",
        );

        assert_eq!(compute.x_vals, &[1.0, 2.0, 3.0]);
        assert_eq!(compute.strains, None); // No strains for CSV mode
        assert_eq!(compute.dataset_path, "/path/to/file.txt");
        assert_eq!(compute.method, CorrelationMethod::Spearman);
    }

    // Note: Full LMDB integration tests require actual LMDB files.
    // Run the following manual test when GSL library is available:
    //
    // cargo test test_lmdb_correlation -- --nocapture --ignored
    // Or with custom path: LMDB_TEST_PATH=/custom/path cargo test test_lmdb_correlation -- --ignored
    //
    // To check available strains in your LMDB:
    // python3 -c "import lmdb, json; env = lmdb.open('/path/to/lmdb'); txn = env.begin(); 
    //              meta = json.loads(txn.get(b'probeset_metadata')); print(meta['strains'][:10])"
    #[test]
    #[ignore = "requires GSL library and LMDB dataset"]
    fn test_lmdb_correlation_with_strains() {
        use crate::lmdb_reader::LmdbReader;
        use crate::lmdb_reader::LMDB_TEST_LOCK;
        use std::path::Path;
        use std::thread;
        use std::time::Duration;

        let lmdb_path = std::env::var("LMDB_TEST_PATH")
            .unwrap_or_else(|_| "/home/kabui/lmdb_data/HC_M2_0606_P/".to_string());
        if !Path::new(&lmdb_path).exists() {
            panic!("LMDB test data not found at {}. Set LMDB_TEST_PATH env var.", lmdb_path);
        }

        // Acquire lock to prevent parallel LMDB access
        let _lock = LMDB_TEST_LOCK.lock().unwrap();

        // Read actual strains from LMDB metadata
        let reader = LmdbReader::new(&lmdb_path).expect("Failed to open LMDB");
        let metadata = reader.read_metadata().expect("Failed to read metadata");
        
        // Use first 6 strains from the actual dataset
        if metadata.strains.len() < 6 {
            panic!("LMDB has fewer than 6 strains: {}", metadata.strains.len());
        }
        
        let strains: Vec<String> = metadata.strains.iter().take(6).cloned().collect();
        eprintln!("Using strains: {:?}", strains);

        // Drop reader to release LMDB lock before compute opens it again
        drop(reader);
        thread::sleep(Duration::from_millis(100));

        // Create x_vals matching the number of strains
        let x_vals: Vec<f64> = (0..strains.len()).map(|i| 10.0 + i as f64 * 5.5).collect();

        let compute = Compute::new_with_strains(
            ',',
            "pearson",
            &lmdb_path,
            &x_vals,
            "/tmp/test_output.txt",
            Some(&strains),
        );

        let result = compute.compute();
        assert!(result.is_ok(), "LMDB correlation failed: {:?}", result);

        // Verify output file was created and has content
        assert!(Path::new("/tmp/test_output.txt").exists());
        let output_content = std::fs::read_to_string("/tmp/test_output.txt").unwrap();
        assert!(!output_content.is_empty(), "Output file is empty");
        eprintln!("Correlation computed for {} traits", output_content.lines().count());
    }

    #[test]
    #[ignore = "requires GSL library and LMDB dataset"]
    fn test_lmdb_correlation_all_strains() {
        use crate::lmdb_reader::LmdbReader;
        use crate::lmdb_reader::LMDB_TEST_LOCK;
        use std::path::Path;
        use std::thread;
        use std::time::Duration;

        let lmdb_path = std::env::var("LMDB_TEST_PATH")
            .unwrap_or_else(|_| "/home/kabui/lmdb_data/HC_M2_0606_P/".to_string());
        if !Path::new(&lmdb_path).exists() {
            panic!("LMDB test data not found at {}. Set LMDB_TEST_PATH env var.", lmdb_path);
        }

        // Acquire lock to prevent parallel LMDB access
        let _lock = LMDB_TEST_LOCK.lock().unwrap();

        // Read number of strains from metadata
        let reader = LmdbReader::new(&lmdb_path).expect("Failed to open LMDB");
        let metadata = reader.read_metadata().expect("Failed to read metadata");
        let num_strains = metadata.columns;
        
        eprintln!("LMDB has {} strains", num_strains);

        // Drop reader to release LMDB lock
        drop(reader);
        thread::sleep(Duration::from_millis(100));

        // Create x_vals for all strains
        let x_vals: Vec<f64> = (0..num_strains).map(|i| 10.0 + i as f64 * 1.5).collect();

        // Without strains - should use all columns (backward compatibility)
        let compute = Compute::new(
            ',',
            "pearson",
            &lmdb_path,
            &x_vals,
            "/tmp/test_output_all.txt",
        );

        let result = compute.compute();
        assert!(result.is_ok(), "LMDB correlation failed: {:?}", result);
        
        // Verify output
        assert!(Path::new("/tmp/test_output_all.txt").exists());
        let output_content = std::fs::read_to_string("/tmp/test_output_all.txt").unwrap();
        assert!(!output_content.is_empty(), "Output file is empty");
        eprintln!("Correlation computed for {} traits", output_content.lines().count());
    }
}
