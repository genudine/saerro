// GENERATED CODE -- Do not edit. Run `cargo run --bin codegen` to regenerate.

use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    static ref VEHICLE_TO_NAME: HashMap<&'static str, &'static str> = HashMap::from([
        ("1", "flash"),
        ("2", "sunderer"),
        ("3", "lightning"),
        ("4", "magrider"),
        ("5", "vanguard"),
        ("6", "prowler"),
        ("7", "scythe"),
        ("8", "reaver"),
        ("9", "mosquito"),
        ("10", "liberator"),
        ("11", "galaxy"),
        ("12", "harasser"),
        ("14", "valkyrie"),
        ("15", "ant"),
        ("160", "ant"),
        ("161", "ant"),
        ("162", "ant"),
        ("1001", "flash"),
        ("1002", "sunderer"),
        ("1004", "magrider"),
        ("1005", "vanguard"),
        ("1007", "scythe"),
        ("1008", "reaver"),
        ("1009", "mosquito"),
        ("1010", "liberator"),
        ("1011", "galaxy"),
        ("1105", "vanguard"),
        ("2010", "flash"),
        ("2033", "javelin"),
        ("2039", "ant"),
        ("2040", "valkyrie"),
        ("2122", "mosquito"),
        ("2123", "reaver"),
        ("2124", "scythe"),
        ("2125", "javelin"),
        ("2129", "javelin"),
        ("2130", "sunderer"),
        ("2131", "galaxy"),
        ("2132", "valkyrie"),
        ("2133", "magrider"),
        ("2134", "vanguard"),
        ("2135", "prowler"),
        ("2136", "dervish"),
        ("2137", "chimera"),
        ("2139", "ant"),
        ("2140", "galaxy"),
        ("2141", "valkyrie"),
        ("2142", "corsair"),
    ]);
    static ref LOADOUT_TO_CLASS: HashMap<&'static str, &'static str> = HashMap::from([
        ("1", "infiltrator"),
        ("3", "light_assault"),
        ("4", "combat_medic"),
        ("5", "engineer"),
        ("6", "heavy_assault"),
        ("7", "max"),
        ("8", "infiltrator"),
        ("10", "light_assault"),
        ("11", "combat_medic"),
        ("12", "engineer"),
        ("13", "heavy_assault"),
        ("14", "max"),
        ("15", "infiltrator"),
        ("17", "light_assault"),
        ("18", "combat_medic"),
        ("19", "engineer"),
        ("20", "heavy_assault"),
        ("21", "max"),
        ("28", "infiltrator"),
        ("29", "light_assault"),
        ("30", "combat_medic"),
        ("31", "engineer"),
        ("32", "heavy_assault"),
        ("45", "max"),
    ]);
}

pub fn vehicle_to_name(vehicle_id: &str) -> String {
    match VEHICLE_TO_NAME.get(&vehicle_id) {
        Some(name) => name.to_string(),
        None => "unknown".to_string(),
    }
}

pub fn loadout_to_class(loadout_id: &str) -> String {
    match LOADOUT_TO_CLASS.get(&loadout_id) {
        Some(name) => name.to_string(),
        None => "unknown".to_string(),
    }
}
