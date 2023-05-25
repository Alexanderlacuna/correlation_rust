use std::cmp::Ordering;
use rayon::prelude::*;
use crate::parser::parse_rows_with_names;
use crate::reader::BufferReader;
use crate::sorter::sort_write_to_file;

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

#[allow(dead_code)]
#[derive(PartialEq, Debug)]
pub struct Compute<'a> {
    x_vals: &'a [f64],
    dataset_path: &'a str,
    method: CorrelationMethod,
    file_delimiter: char,
    output_file: &'a str,
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
        //capitalize method/same cases

        let method = match method.to_lowercase().as_str() {
            "pearson" => CorrelationMethod::Pearson,
            "spearman" => CorrelationMethod::Spearman,
            _ => panic!("method cannot be found"),
        };

        Self {
            x_vals,
            dataset_path,
            method,
            output_file,
            file_delimiter: file_delimeter,
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

      use std::io::{self, BufRead, BufReader};     


        let chunk_size = 1000; // Set the desired chunk size here

         let file = std::fs::File::open(&self.dataset_path)?;

        let reader_1 = BufReader::new(file);

           let chunks: Vec<Vec<String>> = reader_1
        .lines()
        .filter_map(|line| line.ok())
        .collect::<Vec<String>>()
        .chunks(chunk_size)
        .map(|chunk| chunk.to_vec())
        .collect();

        let results:Vec<Vec<(String, f64, f64, i32)>> = chunks 
        .par_iter().map(|chunk|{

              let mut chunk_results = Vec::new();
            for line  in chunk {

                      let ty = parse_rows_with_names(
                    self.x_vals,
                    &line
                        .split(self.file_delimiter)
                        .collect::<Vec<&str>>(),
       
                );
                if ty.x_vals.len() < 4 || ty.y_vals.len() < 4 {
                // minimum number of acceptable trait values for
                // computing the correlations
                continue;
            }

                        let (key_name, parsed_x_val, parsed_y_val) =
                (ty.row_name, ty.x_vals, ty.y_vals);

                 let (rho, p_val) = match self.method {
                    CorrelationMethod::Pearson => Pearson::new(parsed_x_val.len()).correlate(&parsed_x_val, &parsed_y_val),
                    _ => Spearman::new(parsed_x_val.len()).correlate(&parsed_x_val, &parsed_y_val),
                };
                 chunk_results.push((key_name, rho, p_val, parsed_x_val.len() as i32));
              

            }
               chunk_results

        }).collect();

           for chunk_result in  results{
        corr_results.extend(chunk_result);
    }

        corr_results.sort_by(|a, b| {
	    b.1.abs().partial_cmp(&a.1.abs()).unwrap_or_else(|| {
		Ordering::Less})});
        sort_write_to_file(String::from(self.output_file),
            corr_results[0..500].to_vec())
 

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
                    dataset_path: "./notes.txt",
                    output_file: "/output.txt",
                    method: CorrelationMethod::Spearman
                },
                Compute {
                    file_delimiter: ',',
                    x_vals: &[1., 3., 4., 5., 6., 7., 7.],
                    dataset_path: "./note2.txt",
                    output_file: "/output.txt",
                    method: CorrelationMethod::Pearson
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
                dataset_path: "/notes.txt",
                method: CorrelationMethod::Pearson,
                output_file: "/output.txt"
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
}
