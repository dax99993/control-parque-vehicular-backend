use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Department {
    pub id: i32,
    pub name: String,
}
