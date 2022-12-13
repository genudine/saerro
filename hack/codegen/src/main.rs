use std::process;
use lazy_static::lazy_static;
use regex::{Regex, RegexBuilder};
use serde::{Deserialize, Serialize};
use tera::Tera;

#[derive(Deserialize, Serialize, Debug)]
struct LangEn {
    en: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
struct Vehicle {
    vehicle_id: String,
    name: Option<LangEn>,
    propulsion_type: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct VehicleResponse {
    vehicle_list: Vec<Vehicle>,
}

#[derive(Deserialize, Serialize, Debug)]
struct Loadout {
    loadout_id: String,
    code_name: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct ClassesResponse {
    loadout_list: Vec<Loadout>,
}

async fn translators_rs() {
    lazy_static! {
        static ref ALL_VEHICLES: Vec<&'static str> = vec![
            "flash",
            "sunderer",
            "lightning",
            "scythe",
            "vanguard",
            "prowler",
            "reaver",
            "mosquito",
            "galaxy",
            "valkyrie",
            "wasp",
            "deliverer",
            "lodestar",
            "liberator",
            "ant",
            "harasser",
            "dervish",
            "chimera",
            "javelin",
            "corsair",
            "magrider",
        ];
        static ref VEHICLES_RE: Regex =
            RegexBuilder::new(format!("{}", ALL_VEHICLES.join("|")).as_str())
                .case_insensitive(true)
                .build()
                .unwrap();
    }

    let mut tera = Tera::default();

    tera.add_raw_template(
        "translators.rs",
        include_str!("templates/translators.rs.tera"),
    )
    .unwrap();

    let res: VehicleResponse = reqwest::get("https://census.lithafalcon.cc/get/ps2/vehicle")
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let vehicles: Vec<Vehicle> = res
        .vehicle_list
        .into_iter()
        .filter(|item| {
            // filter if no name
            if item.name.is_none() || item.name.as_ref().unwrap().en.is_none() {
                // println!("Skipping vehicle (no name): {:#?}", item);
                return false;
            }

            let name = item.name.as_ref().unwrap().en.as_ref().unwrap();
            if name.contains("Turret") {
                // println!("Skipping vehicle (turret): {:#?}", item);
                return false;
            }

            // filter if not in list
            if !VEHICLES_RE.is_match(name.as_str()) {
                // println!("Skipping vehicle (not tracked): {:#?}", item);
                return false;
            }

            return true;
        })
        .map(|item| {
            // match to regex
            let matched = VEHICLES_RE
                .find(&item.name.as_ref().unwrap().en.as_ref().unwrap())
                .unwrap();

            let name = matched
                .as_str()
                .to_lowercase()
                .replace("wasp", "valkyrie")
                .replace("deliverer", "ant")
                .replace("lodestar", "galaxy");

            Vehicle {
                vehicle_id: item.vehicle_id,
                name: Some(LangEn { en: Some(name) }),
                propulsion_type: item.propulsion_type,
            }
        })
        .collect();

    let res: ClassesResponse = reqwest::get("https://census.lithafalcon.cc/get/ps2/loadout")
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let classes: Vec<Loadout> = res
        .loadout_list
        .into_iter()
        .map(|item| {
            let new_name = item
                .code_name
                .replace(" ", "_")
                .to_lowercase()
                .replace("vs_", "")
                .replace("tr_", "")
                .replace("nso_", "")
                .replace("nc_", "")
                .replace("defector", "max");
            Loadout {
                loadout_id: item.loadout_id.clone(),
                code_name: new_name,
            }
        })
        .collect();

    let mut context = tera::Context::new();
    context.insert("vehicles", &vehicles);
    context.insert("classes", &classes);

    let rendered = tera.render("translators.rs", &context).unwrap();
    let path_raw = format!(
        "{}/../../services/websocket/src/translators.rs",
        env!("CARGO_MANIFEST_DIR")
    );
    let path = std::path::Path::new(path_raw.as_str());

    std::fs::write(path, rendered).unwrap();
    process::Command::new("rustfmt")
        .arg(path)
        .output()
        .expect("failed to execute process");
}

#[tokio::main]
async fn main() {
    translators_rs().await;
}
