use async_graphql::Object;

pub struct Analytics {}

#[Object]
impl Analytics {
    async fn population(&self) -> i32 {
        0
    }
}

#[derive(Default)]
pub struct AnalyticsQuery;

#[Object]
impl AnalyticsQuery {
    async fn analytics(&self) -> Analytics {
        Analytics {}
    }
}
