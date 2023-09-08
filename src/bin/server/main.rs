use crab_messenger::messenger::{
    messenger_server::{Messenger, MessengerServer},
    Message,
};
use tonic::{transport::Server, Request, Response, Status};

#[derive(Default)]
pub struct MyMessenger {}

#[tonic::async_trait]
impl Messenger for MyMessenger {
    async fn chat(&self, request: Request<Message>) -> Result<Response<Message>, Status> {
        println!("Got a request: {:?}", request);

        let reply = Message {
            message: request.into_inner().message,
        };

        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let greeter = MyMessenger::default();

    Server::builder()
        .add_service(MessengerServer::new(greeter))
        .serve(addr)
        .await?;

    Ok(())
}
