use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::NaiveDateTime;

#[derive(Debug, Serialize, Deserialize)]
pub struct Vehicule {
    pub vehicule_id: Uuid,
    pub branch: String,
    pub model: String,
    pub year: i16,
    pub number_plate: String,
    pub short_name: String,
    pub number_card: String,
    pub status: String,
    pub active: bool,
    pub picture: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}


impl Vehicule {
    pub fn is_active(&self) -> bool {
        self.active
    }
    pub fn is_available(&self) -> bool {
        self.status == "available".to_string()
    }
    pub fn is_occupied(&self) -> bool {
        self.status == "occupied".to_string()
    }
    pub fn is_maintenance(&self) -> bool {
        self.status == "maintenance".to_string()
    }

    pub fn update(mut self, update: UpdateVehicule) -> Self {
        self.branch = update.branch.unwrap_or_else(|| self.branch);
        self.model = update.model.unwrap_or_else(|| self.model);
        self.year = update.year.unwrap_or_else(|| self.year);
        self.number_plate= update.number_plate.unwrap_or_else(|| self.number_plate);
        self.short_name= update.short_name.unwrap_or_else(|| self.short_name);
        self.number_card= update.number_card.unwrap_or_else(|| self.number_card);
        self.status= update.status.unwrap_or_else(|| self.status);
        self.active= update.active.unwrap_or_else(|| self.active);
        self.picture= update.picture.unwrap_or_else(|| self.picture);

        self
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewVehicule {
    pub branch: String,
    pub model: String,
    pub year: i16,
    pub number_plate: String,
    pub short_name: String,
    pub number_card: String,
    //pub status: String,
    //pub active: bool,
    //pub picture: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateVehicule {
    pub branch: Option<String>,
    pub model: Option<String>,
    pub year: Option<i16>,
    pub number_plate: Option<String>,
    pub short_name: Option<String>,
    pub number_card: Option<String>,
    pub status: Option<String>,
    pub active: Option<bool>,
    pub picture: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FilteredVehicule {
    pub vehicule_id: Uuid,
    pub branch: String,
    pub model: String,
    pub year: i16,
    pub number_plate: String,
    pub short_name: String,
    pub number_card: String,
    //pub status: String,
    //pub active: bool,
    pub picture: String,
    //pub created_at: NaiveDateTime,
    //pub updated_at: NaiveDateTime,
}

impl FilteredVehicule {
    pub fn from(veh: Vehicule) -> Self {
        Self { 
            vehicule_id: veh.vehicule_id,
            branch: veh.branch,
            model: veh.model,
            year: veh.year,
            number_plate: veh.number_plate,
            short_name: veh.short_name,
            number_card: veh.number_card,
            picture: veh.picture,
        }
    }
}
