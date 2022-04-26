pub mod reader;
use rgsl::{
    randist::t_distribution::{tdist_P, tdist_Q},
    statistics::{correlation, spearman},
};
use assert_approx_eq::assert_approx_eq;

use reader::{
    file_reader,
    add_numbers_2
};


trait Correlation {
    fn correlate(&self,x:&[f64],y:&[f64]) ->(f64,f64);

    
}


pub fn add(a: i32, b: i32) -> i32 {
    a + b
}


#[derive(PartialEq, Debug)]
struct Pearson {
    n: usize,
    degrees_of_freedom:f64,
}



impl Pearson {
    fn new(n:usize) -> Self {

        Self {
            n,
            degrees_of_freedom:(n - 2) as f64,
        }
    }
}




impl Correlation for Pearson {
    fn correlate(&self,x:&[f64],y:&[f64]) ->(f64,f64){

        let corr_coeff = correlation(x, 1, y, 1, self.n);

        //compute p_val two sided

         // P-value (two-sided)
        // Based on R's cor.test method (https://github.com/SurajGupta/r-source/blob/a28e609e72ed7c47f6ddfbb86c85279a0750f0b7/src/library/stats/R/cor.test.R#L21


        let statistic = self.degrees_of_freedom.sqrt() * corr_coeff / (1.0 - corr_coeff.powi(2)).sqrt();

        let p_val = 2.0
        * tdist_P(statistic, self.degrees_of_freedom)
            .min(tdist_Q(statistic, self.degrees_of_freedom));

        (corr_coeff,p_val)
    }
}


#[derive(PartialEq, Debug)]
struct Spearman {
    n: usize,
    degrees_of_freedom:f64,
}


impl Spearman {
    fn new(n:usize) ->Spearman {
        Spearman { n: n, degrees_of_freedom: (n - 2) as f64 }
    }
}

impl Correlation for Spearman {

    fn correlate(&self,x:&[f64],y:&[f64]) -> (f64,f64){

        match  x.len() == y.len() {
            true => println!("success"),
            false => panic!("array should be of the same size")
        };

        //dditional workspace of size 2*n is required in work
        let mut vec = Vec::with_capacity(2 * self.n);
        let workspace: &mut [f64] = vec.as_mut_slice();
        let rho = spearman(x, 1, y, 1, self.n, workspace);

        //p-val two sided

        let t = rho * (self.degrees_of_freedom / ((rho + 1.0) * (1.0 - rho))).sqrt();
        let p_val = 2.0* (tdist_Q(t.abs(), self.degrees_of_freedom));
 
        return (rho,p_val);
    }
}



#[cfg(test)]
mod tests {
    use crate::reader::add_numbers_2;

    use super::*;
    #[test]
    fn test_add_numbers() {
        assert_eq!(add(2,3),5)
    }

    #[test]
    fn test_pearson_obj(){
        let new_pearson = Pearson::new(5);
        assert_eq!(new_pearson,Pearson{n:5, degrees_of_freedom:3.0});
    }
    #[test]
    fn test_pearson_correlation(){
        let new_pearson = Pearson::new(3);
        let (corr_coeff,p_val) = new_pearson.correlate(&[1.,2.,3.,4.,5.] ,&[1.,2.,3.,4.,5.]);

        assert_eq!(format!("{:.2}", corr_coeff),"1.00");
        assert_approx_eq!(p_val,1.341575855,2f64);
    }

    #[test]
    fn test_spearman_obj(){
        let new_spearman = Spearman::new(5);

        assert_eq!(new_spearman,Spearman{n:5, degrees_of_freedom:3.0});

    }
    #[test]
    fn test_spearman_correlation(){
        let new_spearman = Spearman::new(5);

        let (s_rho,p_val) = new_spearman.correlate(&[1.,2.,3.,4.,5.], &[5.,6.,7.,8.,7.]);


        assert_approx_eq!(s_rho,0.079603960396039,2f64);
        assert_approx_eq!(p_val,0.43111687,2f64)


    }

    #[test]
    fn test_add_2(){
        assert_eq!(add_numbers_2(4, 5),9)
    }

    #[test]

    fn test_file_reader_error(){
        assert!(file_reader("./non_file.txt").is_err());
    }


    #[test]

    fn test_file_reader() {
      let expected_results = file_reader("./mock_dataset.txt");

      assert!(expected_results.is_ok())
    }
}