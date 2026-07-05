use std::sync::Arc;

use crate::{
    routing::model::RoutingInfo, transit::model::TransitInfo, transit_router::HasTransitInfo,
};

pub struct State {
    pub routing_info: RoutingInfo,
    pub transit_info: TransitInfo,
}

impl HasTransitInfo for State {
    fn transit_info(&self) -> &TransitInfo {
        &self.transit_info
    }
}

pub type SharedState = Arc<State>;
