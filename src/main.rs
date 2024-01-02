use eframe::NativeOptions;
use tlash::MyApp;
#[tokio::main]
async fn main() {
    eframe::run_native(
        "Tlash",
        NativeOptions::default(),
        Box::new(|_| Box::new(MyApp::new())),
    );
}
