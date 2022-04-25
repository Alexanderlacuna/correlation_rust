use rgsl::{
    randist::t_distribution::{tdist_P, tdist_Q},
    statistics::{correlation, spearman},
};


trait Correlation {
    fn correlate(&self,x:&[f64],y:&[f64]) ->(f64,f64);
}


pub fn add(a: i32, b: i32) -> i32 {
    a + b
}



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
        (1.2,0.1)
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
}