use criterion::{black_box, criterion_group, criterion_main, Criterion,BatchSize};
use correlation_rust::correlations::Compute;



fn criterion_benchmark(c: &mut Criterion) {

    c.bench_function("correlation_rust", |b|b.iter(||{

        let  x_vals = [25.08439 ,72.02225 ,47.56293 ,22.87893 ,14.28721 ,71.84655 ,87.81991 ,84.86824 ,6.72478 ,5.72373 ,73.47078 ,63.74703];
        
        let compute_obj = Compute::new(',',"pearson","/home/kabui/correlation_rust/src/db300.txt",&x_vals);

        compute_obj.compute();
    }));

}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);