use std::env;
use std::time::Instant;
use tonic::{transport::Server, Request, Response, Status};

// Import the generated proto code
pub mod greeter {
    tonic::include_proto!("greeter");
}

// Import the generated client and server traits
use greeter::greeter_client::GreeterClient;
use greeter::greeter_server::{Greeter, GreeterServer};
use greeter::{HelloRequest, HelloReply};

// Implement the Greeter service
#[derive(Debug, Default)]
pub struct MyGreeter {}

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        // println!("Got a request: {:?}", request);

        let reply = HelloReply {
            result: 42.0,
        };

        Ok(Response::new(reply))
    }
}

async fn run_server() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50052".parse()?;
    let greeter = MyGreeter::default();

    println!("Server listening on {}", addr);

    Server::builder()
        .add_service(GreeterServer::new(greeter))
        .serve(addr)
        .await?;

    Ok(())
}

async fn run_client() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = GreeterClient::connect("http://[::1]:50052").await?;

    let request = Request::new(HelloRequest {
        value: 42.0,
    });

    let _response = client.say_hello(request).await?;

    // println!("Response from server: {:?}", _response);

    Ok(())
}

async fn run_benchmark(iterations: u32, num_clients: u32) -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting benchmark with {} iterations across {} clients...", iterations, num_clients);
    let start = Instant::now();
    
    // Calculate iterations per client
    let iterations_per_client = iterations / num_clients;
    
    // Create a vector to hold all client tasks
    let mut handles = Vec::new();
    
    // Spawn tasks for each client
    for client_id in 0..num_clients {
        // Spawn a new task for this client
        let handle = tokio::spawn(async move {
            // Create a new client connection for each task
            let mut client = match GreeterClient::connect("http://[::1]:50052").await {
                Ok(client) => client,
                Err(e) => {
                    eprintln!("Client {} failed to connect: {}", client_id, e);
                    return 0; // Return 0 successful requests
                }
            };
            
            let mut successful_requests = 0;
            
            // Calculate the start index for this client's iterations
            let start_idx = client_id * iterations_per_client;
            
            // Run this client's portion of iterations
            for i in 0..iterations_per_client {
                let request = Request::new(HelloRequest {
                    value: (start_idx + i) as f64,
                });
                
                match client.say_hello(request).await {
                    Ok(_) => successful_requests += 1,
                    Err(e) => {
                        eprintln!("Client {} request failed: {}", client_id, e);
                    }
                }
                
                // Print progress for each client
                if (i + 1) % 1000 == 0 {
                    println!("Client {} completed {} requests", client_id, i + 1);
                }
            }
            
            successful_requests
        });
        
        handles.push(handle);
    }
    
    // Wait for all clients to complete and collect results
    let mut total_successful_requests = 0;
    for handle in handles {
        match handle.await {
            Ok(successful) => total_successful_requests += successful,
            Err(e) => eprintln!("A client task failed: {}", e),
        }
    }
    
    let duration = start.elapsed();
    let total_seconds = duration.as_secs_f64();
    let requests_per_second = total_successful_requests as f64 / total_seconds;
    
    println!("\nBenchmark Results:");
    println!("Total time: {:.2} seconds", total_seconds);
    println!("Successful requests: {} out of {}", total_successful_requests, iterations);
    println!("Average time per request: {:.3} ms", (total_seconds * 1000.0) / total_successful_requests as f64);
    println!("Requests per second: {:.2}", requests_per_second);
    println!("Clients: {}", num_clients);
    println!("Iterations per client: {}", iterations_per_client);
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        println!("Usage: {} [server|client|benchmark [iterations] [num_clients]]", args[0]);
        return Ok(());
    }

    match args[1].as_str() {
        "server" => {
            run_server().await?;
        }
        "client" => {
            run_client().await?;
        }
        "benchmark" => {
            let iterations = if args.len() > 2 {
                args[2].parse().unwrap_or(1000)
            } else {
                20000
            };
            
            let num_clients = if args.len() > 3 {
                args[3].parse().unwrap_or(1)
            } else {
                1
            };
            
            run_benchmark(iterations, num_clients).await?;
        }
        _ => {
            println!("Invalid argument: {}", args[1]);
            println!("Usage: {} [server|client|benchmark [iterations] [num_clients]]", args[0]);
        }
    }

    Ok(())
}
