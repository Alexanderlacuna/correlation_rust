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
| 300000      | 56.98s     |   10.51 s      |
|Paragraph   | Text         |     |               