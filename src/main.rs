use std::env;
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
        println!("Got a request: {:?}", request);

        let reply = HelloReply {
            message: format!("Hello {}!", request.into_inner().name),
        };

        Ok(Response::new(reply))
    }
}

async fn run_server() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let greeter = MyGreeter::default();

    println!("Server listening on {}", addr);

    Server::builder()
        .add_service(GreeterServer::new(greeter))
        .serve(addr)
        .await?;

    Ok(())
}

async fn run_client() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = GreeterClient::connect("http://[::1]:50051").await?;

    let request = Request::new(HelloRequest {
        name: "Tonic".into(),
    });

    let response = client.say_hello(request).await?;

    println!("Response from server: {:?}", response);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        println!("Usage: {} [server|client]", args[0]);
        return Ok(());
    }

    match args[1].as_str() {
        "server" => {
            run_server().await?;
        }
        "client" => {
            run_client().await?;
        }
        _ => {
            println!("Invalid argument: {}", args[1]);
            println!("Usage: {} [server|client]", args[0]);
        }
    }

    Ok(())
}
