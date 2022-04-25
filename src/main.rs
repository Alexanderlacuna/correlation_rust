use rgsl::{
    randist::t_distribution::{tdist_P, tdist_Q},
    statistics::{correlation, spearman},
};

use assert_approx_eq::assert_approx_eq;

trait Correlation {
    fn correlate(&self,x:&[f64],y:&[f64]) ->(f64,f64);
}


pub fn add(a: i32, b: i32) -> i32 {
    a + b
}


#[derive(PartialEq, Debug)]
struct Pearson {
    n: usize,
    degree_of_freedom:f64,
}

impl Pearson {
    fn new(n:usize) -> Self {

        Self {
            n,
            degree_of_freedom:(n - 2) as f64,
        }
    }
}


impl Correlation for Pearson {
    fn correlate(&self,x:&[f64],y:&[f64]) ->(f64,f64){

        let corr_coeff = correlation(x, 1, y, 1, self.n);

        (corr_coeff,0.1)
    }
}

fn main() {

}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_add_numbers() {
        assert_eq!(add(2,3),5)
    }

    #[test]
    fn test_pearson(){
        let new_pearson = Pearson::new(5);
        assert_eq!(new_pearson,Pearson{n:5, degree_of_freedom:3.0});
    }
    #[test]
    fn test_pearson_correlation(){
        let new_pearson = Pearson::new(3);
        let (corr_coeff,p_val) = new_pearson.correlate(&[1.,2.,3.,4.,5.] ,&[1.,2.,3.,4.,5.]);

        assert_eq!(format!("{:.2}", corr_coeff),"1.00");
        assert_eq!(p_val,0.1);
    }
}