use rgsl::{
    randist::t_distribution::{tdist_P, tdist_Q},
    statistics::{correlation, spearman},
};

pub trait Correlation {
    
    fn correlate(&self,x:&[f64],y:&[f64]) ->(f64,f64);   
}

#[derive(PartialEq, Debug)]
pub struct Pearson {
    n: usize,
    degrees_of_freedom:f64,
}

#[derive(PartialEq, Debug)]
struct Spearman {
    n: usize,
    degrees_of_freedom:f64,
}

impl Pearson {
    pub fn new(n:usize) -> Self {

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

impl Correlation for Spearman {

    fn correlate(&self,x:&[f64],y:&[f64]) -> (f64,f64){

        match  x.len() == y.len() {
            true => println!("safe"),
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
