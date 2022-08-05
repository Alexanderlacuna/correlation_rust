# correlation_rust
implementation of correlation with rust


Ensure you have rust toolchain  and cargo installed

run 

``` rust 

cargo build --release     //for optimization 

//or

cargo build  // for general debug


```

### Run Correlation

accepts json file with the following parameters
example 
expected dataset file example see --> ./tests/data/dataset_with_nans.txt

``` json

//example.json

{
    "file_path":"/home/kabui/correlation_rust/tests/data/db300.txt", //bath to dataset file
    "x_vals":"25.08439, 72.02225, 47.56293, 22.87893, 14.28721, 71.84655, 87.81991, 84.86824, 6.72478, 5.72373, 73.47078, 63.74703",
    "sample_values":"bxd1", //do be implemented 
    "method":"pearson",  //method implemented are pearson and spearman
    "file_delimiter":",",  //needed to parse file e.g 12|212|212 delimiter=1
    "output_file":"/home/kabui/correlation_rust/output.txt" //generic path to ouput results
 }


```

``` rust 


cargo run --release example_json


```

### using guix

## Using Guix

```sh

guix shell --container --network nss-certs --development gn-rust-correlation
```

Export environment variables needed by cargo to get dependencies:

```
export SSL_CERT_DIR="${GUIX_ENVIRONMENT}/etc/ssl/certs"
export SSL_CERT_FILE="${SSL_CERT_DIR}/ca-certificates.crt"

```
Now you can run the cargo commands, e.g. to run unit tests:

``` rust 
cargo test
```


- expected results results are is results to written to file contain

   - row name
   - rho value
   - p_val



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
| 1000000       |  118s | 6.04 s|  
| 300000      | 38.06s     |   4.37s      |
|100000   | 13.17s         |    1.36s   |               
| 10000   | 1.21s          | 0.557s|




### Todo


- [x] implementation of pearson and spearman correlation

- [x] benchmarks and tests


- [ ] code optimization e.g use of iterators and lazy computation

- [x] implementation of extern sorter


- [ ] improve on error handling and messages

- [ ] parallel computation

- [ ]  Figure out how to tell cargo to use the declared dependencies, rather than dowloading the dependencies




