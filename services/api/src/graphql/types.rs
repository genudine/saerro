use juniper::graphql_object;
use once_cell::sync::Lazy;
use rocket_db_pools::deadpool_redis::redis::{cmd, pipe};
use std::{
    collections::HashMap,
    ops::Sub,
    time::{Duration, SystemTime},
};

static WORLD_ID_TO_NAME: Lazy<HashMap<&str, &str>> = Lazy::new(|| {
    HashMap::from([
        ("1", "Connery"),
        ("10", "Miller"),
        ("13", "Cobalt"),
        ("17", "Emerald"),
        ("19", "Jaeger"),
        ("40", "SolTech"),
        ("1000", "Genudine"),
        ("2000", "Ceres"),
    ])
});

#[derive(Clone, Debug)]
pub struct World {
    pub world_id: String,
}

#[graphql_object(context = super::Context)]
impl World {
    pub fn id(&self) -> juniper::ID {
        juniper::ID::from(self.world_id.clone())
    }
    pub fn name(&self) -> String {
        WORLD_ID_TO_NAME
            .get(&self.world_id.to_string().as_str())
            .unwrap_or(&"Unknown")
            .to_string()
    }

    pub async fn population(&self, context: &mut super::Context) -> i32 {
        let mut con = (*context).con.get().await.unwrap();
        let id = self.world_id.to_string();

        let filter_timestamp = SystemTime::now()
            .sub(Duration::from_secs(60 * 15))
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let pop: u32 = cmd("ZCOUNT")
            .arg(format!("wp:{}", id))
            .arg(filter_timestamp)
            .arg("+inf")
            .query_async(&mut con)
            .await
            .unwrap();

        pop as i32
    }

    pub async fn faction_population(&self) -> FactionPopulation {
        FactionPopulation {
            world_id: juniper::ID::from(self.world_id.clone()),
        }
    }

    pub async fn vehicles(&self) -> Vehicles {
        Vehicles {
            world_id: juniper::ID::from(self.world_id.clone()),
        }
    }

    pub async fn classes(&self) -> Classes {
        Classes {
            world_id: juniper::ID::from(self.world_id.clone()),
        }
    }
}

pub struct FactionPopulation {
    world_id: juniper::ID,
}

impl FactionPopulation {
    async fn by_faction(&self, context: &super::Context, world_id: String, faction: i32) -> i32 {
        let mut con = (*context).con.get().await.unwrap();

        let filter_timestamp = SystemTime::now()
            .sub(Duration::from_secs(60 * 15))
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        cmd("ZCOUNT")
            .arg(format!("wp:{}/{}", world_id, faction))
            .arg(filter_timestamp)
            .arg("+inf")
            .query_async(&mut con)
            .await
            .unwrap()
    }
}

#[graphql_object(context = super::Context)]
#[graphql(description = "The population of each faction on a world")]
impl FactionPopulation {
    async fn vs(&self, context: &super::Context) -> i32 {
        self.by_faction(context, self.world_id.to_string(), 1).await
    }
    async fn nc(&self, context: &super::Context) -> i32 {
        self.by_faction(context, self.world_id.to_string(), 2).await
    }
    async fn tr(&self, context: &super::Context) -> i32 {
        self.by_faction(context, self.world_id.to_string(), 3).await
    }
    async fn ns(&self, context: &super::Context) -> i32 {
        self.by_faction(context, self.world_id.to_string(), 4).await
    }
}

pub struct Vehicles {
    world_id: juniper::ID,
}

impl Vehicles {
    async fn get_vehicle(
        &self,
        context: &super::Context,
        world_id: String,
        vehicle_name: &str,
    ) -> i32 {
        let mut con = (*context).con.get().await.unwrap();

        let filter_timestamp = SystemTime::now()
            .sub(Duration::from_secs(60 * 15))
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        cmd("ZCOUNT")
            .arg(format!("v:{}/{}", world_id, vehicle_name))
            .arg(filter_timestamp)
            .arg("+inf")
            .query_async(&mut con)
            .await
            .unwrap()
    }
}

#[graphql_object(context = super::Context)]
#[graphql(description = "The count of active vehicles on a world")]
impl Vehicles {
    // Transporters
    async fn flash(&self, context: &super::Context) -> i32 {
        self.get_vehicle(context, self.world_id.to_string(), "flash")
            .await
    }
    async fn sunderer(&self, context: &super::Context) -> i32 {
        self.get_vehicle(context, self.world_id.to_string(), "sunderer")
            .await
    }
    async fn ant(&self, context: &super::Context) -> i32 {
        self.get_vehicle(context, self.world_id.to_string(), "ant")
            .await
    }
    async fn harasser(&self, context: &super::Context) -> i32 {
        self.get_vehicle(context, self.world_id.to_string(), "harasser")
            .await
    }
    async fn javelin(&self, context: &super::Context) -> i32 {
        self.get_vehicle(context, self.world_id.to_string(), "javelin")
            .await
    }

    // Tanks
    async fn lightning(&self, context: &super::Context) -> i32 {
        self.get_vehicle(context, self.world_id.to_string(), "lightning")
            .await
    }
    async fn prowler(&self, context: &super::Context) -> i32 {
        self.get_vehicle(context, self.world_id.to_string(), "prowler")
            .await
    }
    async fn vanguard(&self, context: &super::Context) -> i32 {
        self.get_vehicle(context, self.world_id.to_string(), "vanguard")
            .await
    }
    async fn magrider(&self, context: &super::Context) -> i32 {
        self.get_vehicle(context, self.world_id.to_string(), "magrider")
            .await
    }
    async fn chimera(&self, context: &super::Context) -> i32 {
        self.get_vehicle(context, self.world_id.to_string(), "chimera")
            .await
    }

    // Air
    async fn mosquito(&self, context: &super::Context) -> i32 {
        self.get_vehicle(context, self.world_id.to_string(), "mosquito")
            .await
    }
    async fn liberator(&self, context: &super::Context) -> i32 {
        self.get_vehicle(context, self.world_id.to_string(), "liberator")
            .await
    }
    async fn galaxy(&self, context: &super::Context) -> i32 {
        self.get_vehicle(context, self.world_id.to_string(), "galaxy")
            .await
    }
    async fn valkyrie(&self, context: &super::Context) -> i32 {
        self.get_vehicle(context, self.world_id.to_string(), "valkyrie")
            .await
    }
    async fn reaver(&self, context: &super::Context) -> i32 {
        self.get_vehicle(context, self.world_id.to_string(), "reaver")
            .await
    }
    async fn scythe(&self, context: &super::Context) -> i32 {
        self.get_vehicle(context, self.world_id.to_string(), "scythe")
            .await
    }
    async fn dervish(&self, context: &super::Context) -> i32 {
        self.get_vehicle(context, self.world_id.to_string(), "dervish")
            .await
    }
}

pub struct Classes {
    pub world_id: juniper::ID,
}

impl Classes {
    async fn get_class(&self, context: &super::Context, world_id: String, class_name: &str) -> i32 {
        let mut con = (*context).con.get().await.unwrap();

        let filter_timestamp = SystemTime::now()
            .sub(Duration::from_secs(60 * 15))
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        cmd("ZCOUNT")
            .arg(format!("c:{}/{}", world_id, class_name))
            .arg(filter_timestamp)
            .arg("+inf")
            .query_async(&mut con)
            .await
            .unwrap()
    }
}

#[graphql_object(context = super::Context)]
#[graphql(description = "The count of active classes on a world")]
impl Classes {
    async fn infiltrator(&self, context: &super::Context) -> i32 {
        self.get_class(context, self.world_id.to_string(), "infiltrator")
            .await
    }
    async fn light_assault(&self, context: &super::Context) -> i32 {
        self.get_class(context, self.world_id.to_string(), "light_assault")
            .await
    }
    async fn combat_medic(&self, context: &super::Context) -> i32 {
        self.get_class(context, self.world_id.to_string(), "combat_medic")
            .await
    }
    async fn engineer(&self, context: &super::Context) -> i32 {
        self.get_class(context, self.world_id.to_string(), "engineer")
            .await
    }
    async fn heavy_assault(&self, context: &super::Context) -> i32 {
        self.get_class(context, self.world_id.to_string(), "heavy_assault")
            .await
    }
    async fn max(&self, context: &super::Context) -> i32 {
        self.get_class(context, self.world_id.to_string(), "max")
            .await
    }
}

#[derive(juniper::GraphQLEnum)]
enum WebsocketState {
    #[graphql(
        description = "Using Nanite Systems manifold. This is the best possible running state."
    )]
    Primary,

    #[graphql(
        description = "Using backup Daybreak Games manifold. This means the primary socket hasn't recieved events for at least 60 seconds."
    )]
    Backup,

    #[graphql(description = "Both event processors are down. This is bad.")]
    Down,
}

#[derive(juniper::GraphQLEnum)]
enum UpDown {
    #[graphql(description = "Checks have passed.")]
    Up,

    #[graphql(description = "Checks have failed. This is bad.")]
    Down,
}

pub struct Health {}

impl Health {
    async fn get_heartbeat(context: &super::Context, pair: &str) -> WebsocketState {
        let mut con = (*context).con.get().await.unwrap();

        let res: Result<i32, _> = cmd("GET")
            .arg(format!("heartbeat:{}:primary", pair))
            .query_async(&mut *con)
            .await;
        match res {
            Ok(_) => WebsocketState::Primary,
            Err(_) => {
                let res: Result<i32, _> = cmd("GET")
                    .arg(format!("heartbeat:{}:backup", pair))
                    .query_async(&mut con)
                    .await;
                match res {
                    Ok(_) => WebsocketState::Backup,
                    Err(_) => WebsocketState::Down,
                }
            }
        }
    }
}

#[graphql_object(context = super::Context)]
#[graphql(description = "Saerro's self-checks. Down is universally bad.")]
impl Health {
    #[graphql(description = "Checks PC event processors for its running state.")]
    async fn pc(context: &super::Context) -> WebsocketState {
        Health::get_heartbeat(context, "pc").await
    }

    #[graphql(description = "Checks PS4 US event processors for its running state.")]
    async fn ps4us(context: &super::Context) -> WebsocketState {
        Health::get_heartbeat(context, "ps4us").await
    }

    #[graphql(description = "Checks PS4 EU event processors for its running state.")]
    async fn ps4eu(context: &super::Context) -> WebsocketState {
        Health::get_heartbeat(context, "ps4eu").await
    }

    #[graphql(description = "Is our datastore working?")]
    async fn redis(context: &super::Context) -> UpDown {
        let mut con = (*context).con.get().await.unwrap();

        let res: Result<String, _> = cmd("PING").query_async(&mut con).await;

        match res {
            Ok(_) => UpDown::Up,
            Err(_) => UpDown::Down,
        }
    }
}
