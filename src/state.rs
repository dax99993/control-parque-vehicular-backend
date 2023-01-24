use crate::database::{DbPool, establish_connection};

/// This struct defines the State for a web app
///
/// # Attributes
/// * pool (DbPool): the database pool
#[derive(Debug, Clone)]
pub struct State {
    pub pool: DbPool,
}

impl State {

    /// The Constructor/Initializer for the State struct
    ///
    /// # Arguments
    /// None
    ///
    /// # Returns
    /// (State): The constructed State struct
    ///
    pub async fn init() -> Self {
        let state = State { pool: establish_connection().await };
        debug!("{:?}", state);

        state
    }
}

/// Type defition for State in ?
pub type AppStateRaw = std::sync::Arc<State>;
/// Type defition for Database Appstate in actix web
pub type AppState = actix_web::web::Data<State>;

