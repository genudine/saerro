import fetch from "node-fetch";

const vehicles_hashmap = async () => {
  const req = await fetch("https://census.lithafalcon.cc/get/ps2/vehicle");
  const resp = await req.json();

  const relevantVehicles = [
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
    "liberator",
    "ant",
    "harasser",
    "dervish",
    "chimera",
    "javelin",
    "corsair",
    "magrider",
  ];

  const matcher = new RegExp(`\\b${relevantVehicles.join("|")}\\b`, "i");

  return resp.vehicle_list
    .reduce((acc, vehicle) => {
      if (vehicle.name?.en) {
        let result = vehicle.name.en.match(matcher);
        if (result) {
          acc.push(`("${vehicle.vehicle_id}", "${result[0].toLowerCase()}")`);
        }
      }

      return acc;
    }, [])
    .filter((v) => !!v);
};

const class_hashmap = async () => {
  const req = await fetch("https://census.lithafalcon.cc/get/ps2/loadout");
  const resp = await req.json();

  return resp.loadout_list.map(
    (loadout) =>
      `("${loadout.loadout_id}", "${loadout.code_name
        .toLowerCase()
        .replace(/\btr|nc|vs|nso\b/, "")
        .trim()
        .replace("defector", "max")
        .replace(/ /g, "_")}")`
  );
};

console.log(`// GENERATED CODE -- Do not edit. Run \`node hack/codegen/codegen.js > services/websocket/src/translators.rs\`  to regenerate.

use once_cell::sync::Lazy;
use std::collections::HashMap;

static VEHICLE_TO_NAME: Lazy<HashMap<&str, &str>> = Lazy::new(|| {
    HashMap::from([
        ${(await vehicles_hashmap()).join(",\n        ")},
    ])
});

pub fn vehicle_to_name(vehicle_id: &str) -> String {
    match VEHICLE_TO_NAME.get(&vehicle_id) {
        Some(name) => name.to_string(),
        None => "unknown".to_string(),
    }
}

static LOADOUT_TO_CLASS: Lazy<HashMap<&str, &str>> = Lazy::new(|| {
    HashMap::from([
        ${(await class_hashmap()).join(",\n        ")},
    ])
});

pub fn loadout_to_class(loadout_id: &str) -> String {
    match LOADOUT_TO_CLASS.get(&loadout_id) {
        Some(name) => name.to_string(),
        None => "unknown".to_string(),
    }
}`);
