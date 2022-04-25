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
        (1.2,2.1)
    }
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
    fn test_spearman(){
        let new_spearman = Spearman::new(5);

        assert_eq!(new_spearman,Spearman{n:5, degrees_of_freedom:3.0});

    }
}