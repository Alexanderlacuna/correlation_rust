# correlation_rust
implementation of correlation with rust


Ensure you have rust toolchain  and cargo installed

run 


``` rust
cargo build


```

### Run Correlation

accepts json file with the following parameters
example 
expected dataset file example see --> ./tests/data/dataset_with_nans.txt
``` json

//example.json
{
    "file_path":"/home/kabui/correlation_rust/tests/data/matrix_80.txt", //dataset path
    "x_vals":"25.08439, 72.02225, 47.56293, 22.87893, 14.28721, 71.84655, 87.81991, 84.86824, 6.72478, 5.72373, 73.47078, 63.74703", //x-vals primary values
    "method":"pearson",  //spearman or pearson
    "file_delimiter":","  //need to parse file e.g "12| 212| 212|" delimiter=|

}

```

``` rust 


cargo run example_json


```
- expected results results are 

 ``` rust
   vec![("y_name","rho","pval")]

```


### Tests


``` rust


 cargo test  // to run the unittests


```


### Benchmark

``` rust

cargo bench

```

### Performance

performance comparison with python scipy stats
 

- this was conducted using same dataset and 16 column matrix




| number of rows      | python | rust |
| ----------- | ----------- | ---------|
| 300000      | 38.06s     |   4.37s      |
|100000   | 13.17s         |    1.36s   |               





### Todo


- [x] implementation of pearson and spearman correlation

- [x] benchmarks and tests

- [ ] code optimization e.g use of iterators and lazy computation

- [ ] improve on error handling and messages

- [ ] parallel computation



