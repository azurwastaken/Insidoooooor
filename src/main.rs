mod sniffooor;

#[tokio::main]
async fn main() {
    sniffooor::sniffa().await;
}