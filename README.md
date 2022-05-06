# correlation_rust
implementation of correlation with rust


Ensure you have rust toolchain  and cargo installed

run 


``` rust
cargo build


```

### Run Correlation

accepts a dataset as text file

``` rust 



Correlation::new(
      dataset_path:&str, // path to dataset file containing y_vals
      file_delimiter: char //example (,| ," ")
      x_vals : &[f64] //contains the primary values
      method: &str , // either pearson or spearman

).compute()



```
- results are 

 ``` rust
   vec![("y_name","rho","pval")]

```


### Tests


``` rust


 cargo test  // to run the unittests


```