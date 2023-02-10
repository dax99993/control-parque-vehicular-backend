#[macro_use]
extern crate nonblock_logger;


use control_parque_vehicular::configuration::get_configuration;
use control_parque_vehicular::startup::Application;




#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Logger
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
    
    // Get Environmental variables
    let configuration = get_configuration().expect("Failed to read configuration.");
    let application = Application::build(configuration).await?;

    application.run_until_stopped().await?;

    Ok(())
}
