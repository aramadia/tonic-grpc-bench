use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use tonic::Request;
use std::time::Duration;
use tokio::runtime::Runtime;

// Import the generated proto code
pub mod greeter {
    tonic::include_proto!("greeter");
}

use greeter::greeter_client::GreeterClient;
use greeter::HelloRequest;

async fn benchmark_say_hello(num_calls: u32) {
    let mut client = GreeterClient::connect("http://[::1]:50052")
        .await
        .expect("Failed to connect to server");
    
    for i in 0..num_calls {
        let request = Request::new(HelloRequest {
            value: i as f64,
        });
        
        let _response = client.say_hello(request)
            .await
            .expect("Failed to call say_hello");
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("grpc_calls");
    group.measurement_time(Duration::from_secs(10));
    
    let num_calls = 1000;
    group.bench_with_input(
        BenchmarkId::new("say_hello", num_calls), 
        &num_calls, 
        |b, &num_calls| {
            b.iter(|| {
                rt.block_on(benchmark_say_hello(num_calls))
            });
        }
    );
    
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
