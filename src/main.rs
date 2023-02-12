use control_parque_vehicular::configuration::get_configuration;
use control_parque_vehicular::startup::Application;
use control_parque_vehicular::telemetry::{init_subscriber, get_subscriber};




#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Logger
    //std::env::set_var("RUST_LOG", "debug");
    //env_logger::init();
    let subscriber = get_subscriber("contro-parque-vehicular".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);
    
    // Get Environmental variables
    let configuration = get_configuration().expect("Failed to read configuration.");
    let application = Application::build(configuration).await?;

    application.run_until_stopped().await?;

    Ok(())
}
