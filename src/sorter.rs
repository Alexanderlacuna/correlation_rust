// implementation of extern sorter with rust

//https://betterprogramming.pub/how-to-sort-a-20g-file-in-rust-12abfffbd92b
use std::cmp::Ordering;
use std::fs::File;
use std::io::{BufWriter, Write};
//use extsort::*;

const BUFFER_CAPACITY: usize = 4_000_000_000;
//struct corr_results

#[derive(Debug, PartialEq, PartialOrd)]
struct CorrResults(String, f64, f64, i32);

pub fn sort_write_to_file(
    filename: String,
    mut v: Vec<(String, f64, f64, i32)>,
) -> std::io::Result<String> {
    File::create(filename.clone())?;

    custom_float_sorter(&mut v);

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

pub fn custom_float_sorter(v: &mut [(String, f64, f64, i32)]) {
    v.sort_by(|a, b| {
        let x = a.1;
        let y = b.1;

        // NaNs always last
        if x.is_nan() && y.is_nan() {
            return Ordering::Equal;
        }
        if x.is_nan() {
            return Ordering::Greater;
        }
        if y.is_nan() {
            return Ordering::Less;
        }

        // Sort by magnitude (|ρ|) descending, including ±∞
        y.abs().partial_cmp(&x.abs()).unwrap()
    });
}

pub fn float_sorter(unsorted: &mut [f64]) {
    //only  for test case
    // custom function to sort floats with nan and inf descending order
    unsorted.sort_by(
        |&a, &b| match (a.is_nan() | a.is_infinite(), b.is_nan() | b.is_infinite()) {
            (true, true) => Ordering::Equal,
            (true, false) => Ordering::Greater,
            (false, true) => Ordering::Less,
            (false, false) => b.abs().partial_cmp(&a.abs()).unwrap(),
        },
    );
}

mod tests {
    use super::custom_float_sorter;

    #[test]
    fn test_basic_sorting_by_magnitude() {
        let mut data = vec![
            ("a".to_string(), 3.0, 0.0, 1),
            ("b".to_string(), -4.5, 0.0, 2),
            ("c".to_string(), 1.2, 0.0, 3),
        ];
        custom_float_sorter(&mut data);
        assert_eq!(
            data,
            vec![
                ("b".to_string(), -4.5, 0.0, 2),
                ("a".to_string(), 3.0, 0.0, 1),
                ("c".to_string(), 1.2, 0.0, 3),
            ]
        );
    }

    #[test]
    fn test_negative_numbers_by_magnitude() {
        let mut data = vec![
            ("x".to_string(), -1.0, 0.0, 1),
            ("y".to_string(), -5.0, 0.0, 2),
            ("z".to_string(), -3.0, 0.0, 3),
        ];
        custom_float_sorter(&mut data);
        assert_eq!(
            data,
            vec![
                ("y".to_string(), -5.0, 0.0, 2),
                ("z".to_string(), -3.0, 0.0, 3),
                ("x".to_string(), -1.0, 0.0, 1),
            ]
        );
    }

    #[test]
    fn test_nan_handling() {
        let mut data = vec![
            ("a".to_string(), f64::NAN, 0.0, 1),
            ("b".to_string(), 2.0, 0.0, 2),
            ("c".to_string(), -4.0, 0.0, 3),
            ("d".to_string(), f64::NAN, 0.0, 4),
        ];
        custom_float_sorter(&mut data);

        // NaN values should be at the end
        assert!(data[2].1.is_nan() || data[3].1.is_nan());
        assert!(data[2].1.is_nan() || data[3].1.is_nan());
    }

    #[test]
    fn test_infinity_handling() {
        let mut data = vec![
            ("a".to_string(), f64::INFINITY, 0.0, 1),
            ("b".to_string(), -3.0, 0.0, 2),
            ("c".to_string(), f64::NEG_INFINITY, 0.0, 3),
        ];
        custom_float_sorter(&mut data);
        assert_eq!(
            data,
            vec![
                ("a".to_string(), f64::INFINITY, 0.0, 1), // largest magnitude
                ("c".to_string(), f64::NEG_INFINITY, 0.0, 3),
                ("b".to_string(), -3.0, 0.0, 2),
            ]
        );
    }

    #[test]
    fn test_mixed_nan_infinity_values_by_magnitude() {
        let mut data = vec![
            ("a".to_string(), f64::INFINITY, 0.0, 1),
            ("b".to_string(), 3.0, 0.0, 2),
            ("c".to_string(), -f64::INFINITY, 0.0, 3),
            ("d".to_string(), f64::NAN, 0.0, 4),
            ("e".to_string(), -2.5, 0.0, 5),
            ("f".to_string(), f64::NAN, 0.0, 6),
        ];
        custom_float_sorter(&mut data);

        // Expected by magnitude: ±∞ first, then 3.0, 2.5, NaN last
        assert!(data[0].1.is_infinite());
        assert!(data[1].1.is_infinite());
        assert_eq!(data[2].1, 3.0);
        assert_eq!(data[3].1, -2.5);
        assert!(data[4].1.is_nan());
        assert!(data[5].1.is_nan());
    }
}
