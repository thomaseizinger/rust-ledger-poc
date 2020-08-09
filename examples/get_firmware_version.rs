#[tokio::main]
async fn main() {
    let transport = HidTransport::new().unwrap();
    let (_, _, v) = get_firmware_version(&transport).await.unwrap();
    println!("firmware version: {}", v);
}
