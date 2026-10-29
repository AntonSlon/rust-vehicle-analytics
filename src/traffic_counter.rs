use core::error;
use std::{collections::HashMap, fs::File, io::Write};
use edgefirst_tracker::Uuid;
use serde_json::json;

use crate::util::transform_uiud;

pub struct TrafficCounter {
    cars: HashMap<Uuid, u32>,
    bicycles: HashMap<Uuid, u32>,
    motorcycles: HashMap<Uuid, u32>,
    bus: HashMap<Uuid, u32>,
    truck: HashMap<Uuid, u32>,
    //person: HashMap<Uuid, u32>,
}

impl TrafficCounter {
    pub fn new() -> Self {
        Self {
            cars: HashMap::new(),
            bicycles: HashMap::new(),
            motorcycles: HashMap::new(),
            bus: HashMap::new(),
            truck: HashMap::new(),
            //person: HashMap::new(),
        }
    }

    pub fn add_vehicle(&mut self, id: u32, uuid: edgefirst_tracker::Uuid, class_id_name: &str) {
        match class_id_name {
            "Car" => {
                if !self.cars.contains_key(&uuid) {
                    self.cars.insert(uuid, id);
                }
            },
            "Bicycle" => {
                if !self.bicycles.contains_key(&uuid) {
                    self.bicycles.insert(uuid, id);
                }
            }
            "Motorcycle" => {
                if !self.motorcycles.contains_key(&uuid) {
                    self.motorcycles.insert(uuid, id);
                }
            },
            "Bus" => {
                if !self.bus.contains_key(&uuid) {
                    self.bus.insert(uuid, id);
                }
            },
            // "Person" => {
            //     if !self.person.contains_key(&uuid) {
            //         self.person.insert(uuid, id);
            //     }
            // },
            "Truck" => {
                if !self.truck.contains_key(&uuid) {
                    self.truck.insert(uuid, id);
                }
            },
            _ => {}

        }
    }

    pub fn total_cars(&self) -> usize {
        self.cars.len()
    }

    pub fn total_bicycles(&self) -> usize {
        self.bicycles.len()
    }

    pub fn total_motorcycles(&self) -> usize {
        self.motorcycles.len()
    }

    pub fn total_bus(&self) -> usize {
        self.bus.len()
    }

    pub fn total_truck(&self) -> usize {
        self.truck.len()
    }

    // pub fn total_person(&self) -> usize {
    //     self.person.len()
    // }

    pub fn total_vehicles(&self) -> usize {
        self.cars.len() + self.motorcycles.len() + self.bus.len() + self.truck.len()
    }

    pub fn save_report(&self) -> Result<(), Box<dyn error::Error>> {
        let cars = transform_uiud(&self.cars);
        let bicycles= transform_uiud(&self.bicycles);
        let motorcycles = transform_uiud(&self.motorcycles);
        let bus = transform_uiud(&self.bus);
        let truck = transform_uiud(&self.truck);
        //let person = transform_uiud(&self.person);
        
        let report = json!({
            "Cars": cars,
            "bicycles": &bicycles,
            "Motorcycles": &motorcycles,
            "Buses": &bus,
            "Trucks": &truck,
            //S"Persons": &person
        });

        let json_string = serde_json::to_string_pretty(&report)?;
        let mut file = File::create("./src/report/report.json")?;
        file.write_all(json_string.as_bytes())?;

        Result::Ok(())
    }
}

