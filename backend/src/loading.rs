use std::io;

use crate::{
    routing::{model::RoutingInfo, osm},
    transit::model::TransitInfo,
};

pub fn load_routing_info() -> io::Result<RoutingInfo> {
    let lesser_poland_by_distance = osm::load_routing_network("osm/LESSER_POLAND")?;

    let mut routing_info = RoutingInfo::new();
    routing_info.insert(
        "Lesser Poland (by distance)".to_owned(),
        lesser_poland_by_distance,
    );

    Ok(routing_info)
}

pub fn load_transit_info() -> io::Result<TransitInfo> {
    // let gtfs_t = load_gtfs(Path::new("gtfs/KRK/T"));
    // let gtfs_a = load_gtfs(Path::new("gtfs/KRK/A"));
    // let gtfs_m = load_gtfs(Path::new("gtfs/KRK/M"));

    let mut _transit_info = TransitInfo::new();
    // transit_info.add_gtfs(&gtfs_t);
    // transit_info.add_gtfs(&gtfs_a);
    // transit_info.add_gtfs(&gtfs_m);

    Ok(_transit_info)
}
