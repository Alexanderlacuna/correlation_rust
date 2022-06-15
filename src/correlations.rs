use crate::parser::parse_rows_with_names;
use crate::sorter::sort_write_to_file;
use crate::reader::BufferReader;

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
            true => println!("safe"),
            false => panic!("array should be of the same size"),
        };

        //dditional workspace of size 2*n is required in work
        let mut vec = Vec::with_capacity(2 * self.n);
        let workspace: &mut [f64] = vec.as_mut_slice();
        let rho = spearman(x, 1, y, 1, self.n, workspace);

        //p-val two sided

        let t = rho * (self.degrees_of_freedom / ((rho + 1.0) * (1.0 - rho))).sqrt();
        let p_val = 2.0 * (tdist_Q(t.abs(), self.degrees_of_freedom));

        return (rho, p_val);
    }
}

#[allow(dead_code)]
#[derive(PartialEq, Debug)]
pub struct Compute<'a> {
    x_vals: &'a [f64],
    dataset_path: &'a str,
    method: CorrelationMethod,
    file_delimiter: char,
}
#[allow(dead_code)]
impl<'a> Compute<'a> {
    pub fn new(
        file_delimeter: char,
        method: &str,
        dataset_path: &'a str,
        x_vals: &'a [f64],
    ) -> Self {
        //capitalize method/same cases

        let method = match method.to_lowercase().as_str() {
            "pearson" => CorrelationMethod::Pearson,
            "spearman" => CorrelationMethod::Spearman,
            _ => panic!("method cannot be found"),
        };

        Self {
            x_vals: x_vals,
            dataset_path: dataset_path,
            method: method,
            file_delimiter: file_delimeter,
        }
    }

    pub fn filter_top(result: Vec<(f64, f64)>, n: Option<usize>) -> Vec<(f64, f64)> {
        match n {
            Some(n_top) => result,

            None => result,
        }
    }

    pub fn sorter(results: &mut Vec<(f64, f64)>) {
        //naive sorter
        let sorted_results = results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    }

    pub fn sorter_with_keys(results: &mut Vec<(String, f64, f64)>) {
        let sorted_results = results.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
    }

    pub fn compute(&self) -> Vec<(String, f64, f64)> {
        //read from file);

        let mut corr_results: Vec<(String, f64, f64)> = Vec::new();

        let reader = BufferReader::new(self.dataset_path);

        match reader {
            Ok(mut buffer_read) => {
                let mut n_string = String::new();

                //assumption made is that first row doesn't contain an empty value

                let column_names = match buffer_read.read_line(&mut n_string) {
                    Some(row) => row
                        .unwrap()
                        .split(self.file_delimiter)
                        .collect::<Vec<&str>>(),

                    None => {
                        //no content in file

                        panic!("expected a row")
                    }
                };

                //create file from here

                while let Some(val) = buffer_read.read_line(&mut n_string) {
                    if let Ok(array_new_val) = val {
                        //let (parsed_x_val, parsed_y_val) = parse_rows(
                        //    self.x_vals,
                        //    &array_new_val
                        //        .split(self.file_delimiter)
                        //        .collect::<Vec<&str>>(),
                        //);

                        let ty = parse_rows_with_names(
                            self.x_vals,
                            &array_new_val
                                .split(self.file_delimiter)
                                .collect::<Vec<&str>>(),
                        );

                        let (key_name, parsed_x_val, parsed_y_val) =
                            (ty.row_name, ty.x_vals, ty.y_vals);

                        if self.method == CorrelationMethod::Pearson {
                            let (rho, p_val) = Pearson::new(parsed_x_val.len())
                                .correlate(&parsed_x_val, &parsed_y_val);

                            corr_results.push((key_name, rho, p_val));

                            //writeln!(file, "{},{},{}",key_name,rho,p_val);
                        } else {
                            let (rho, p_val) = Spearman::new(parsed_x_val.len())
                                .correlate(&parsed_x_val, &parsed_y_val);

                            corr_results.push((key_name, rho, p_val));

                            //writeln!(file, "{},{},{}", key_name,rho,p_val);
                        }
                    }
                }
            }

            Err(err) => panic!("an error ocurrexxxxxxxxxxxxxxd {:?}", err),
        }

        //naive implementation try extern sorting could save 3 seconds


        corr_results.sort_by(|a, b| b.1.abs().partial_cmp(&a.1.abs()).unwrap());

       // corr_results.sort_by(|a,b|b.1.abs().partial_cmp(&a.1.abs()).unwrap());
       sort_write_to_file(String::from("sorted_results.txt"), corr_results).unwrap();
       
        return vec![]
    }
}
#[derive(PartialEq, Debug)]
enum CorrelationMethod {
    Pearson,
    Spearman,
}

#[cfg(test)]

mod test {

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
            Compute::new(',', "spearman", "./notes.txt", &x_vals),
            Compute::new(',', "pearson", "./note2.txt", &x_vals),
        ];

        assert_eq!(
            [
                Compute {
                    file_delimiter: ',',
                    x_vals: &[1., 3., 4., 5., 6., 7., 7.],
                    dataset_path: "./notes.txt",
                    method: CorrelationMethod::Spearman
                },
                Compute {
                    file_delimiter: ',',
                    x_vals: &[1., 3., 4., 5., 6., 7., 7.],
                    dataset_path: "./note2.txt",
                    method: CorrelationMethod::Pearson
                }
            ],
            new_compute
        )
    }

    #[test]
    #[should_panic]
    fn test_compute_panics() {
        let new_compute = Compute::new(',', "unknown", "/notes.txt", &[1.2, 1.4, 1.5]);
        assert_eq!(
            new_compute,
            Compute {
                file_delimiter: ',',
                x_vals: &[1.2, 1.4, 1.5],
                dataset_path: "/notes.txt",
                method: CorrelationMethod::Pearson
            }
        )
    }

    #[test]
    fn test_compute() {
        let compute_obj = Compute::new(
            ',',
            "pearson",
            "/home/kabui/correlation_rust/tests/data/mock_dataset.txt",
            &[12., 15., 11., 11., 16., 11., 8., 7.],
        );

        let corr_results = compute_obj.compute();

        let expected_results = vec![
            (1.0, 0.0),
            (-0.2550, 0.6787),
            (-0.2168, 0.7260),
            (0.3941, 0.5115),
        ];

        for (index, (_key_name, rho, pval)) in corr_results.iter().enumerate() {
            let (exp_corr, exp_pval) = &(expected_results[index]);

            assert_approx_eq!(rho, exp_corr, 2f64);
            assert_approx_eq!(pval, exp_pval, 2f64);
        }
    }

    #[test]

    fn test_parse_f64() {
        let data = "9. ,5. ,0. ,7. ,6. ,1. ,5. ,0.\n";

        let y = "25.08439 ,72.02225 ,47.56293 ,22.87893 ,14.28721 ,71.84655 ,87.81991 ,84.86824 ,6.72478 ,5.72373 ,73.47078 ,63.74703";

        let expected_results = vec![9., 5., 0., 7., 6., 1., 5., 0.];

        let results: Vec<f64> = data
            .split(",")
            .map(|f_str| f_str.trim().parse::<f64>().expect("parse failed"))
            .collect();

        let u: Vec<f64> = y
            .split(",")
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

    fn test_huge_dataset() {
        let x_vals = [
            25.08439, 72.02225, 47.56293, 22.87893, 14.28721, 71.84655, 87.81991, 84.86824,
            6.72478, 5.72373, 73.47078, 63.74703,
        ];

        //for tests only

        let compute_obj = Compute::new(
            ',',
            "pearson",
            "/home/kabui/correlation_rust/tests/data/matrix_80.txt",
            &x_vals,
        );

        assert_eq!(
            vec![(String::from("hello"), 1.2, 1.5)],
            compute_obj.compute()
        );
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
