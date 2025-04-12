use serde::{Serialize, Deserialize};
use rmp_serde::{Serializer, Deserializer};
use std::io::Cursor;
use std::error::Error;

/// Root response structure
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct RouteResponse {
    pub code: String,
    pub routes: Vec<Route>,
    pub waypoints: Vec<Waypoint>,
}

/// Route information
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Route {
    pub legs: Vec<Leg>,
    pub weight_name: String,
    pub weight: f64,
    pub duration: f64,
    pub distance: f64,
}

/// Leg of a route (a segment between waypoints)
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Leg {
    pub steps: Vec<Step>,
    pub summary: String,
    pub weight: f64,
    pub duration: f64,
    pub distance: f64,
}

/// Step in a leg (a navigation instruction)
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Step {
    pub geometry: String,
    pub maneuver: Maneuver,
    pub mode: String,
    pub driving_side: Option<String>,
    pub name: String,
    pub intersections: Vec<Intersection>,
    pub weight: f64,
    pub duration: f64,
    pub distance: f64,
    pub ref_: Option<String>,
    #[serde(rename = "ref")]
    pub ref_field: Option<String>,
}

/// Maneuver information
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Maneuver {
    pub bearing_after: i32,
    pub bearing_before: i32,
    pub location: [f64; 2],
    pub modifier: String,
    #[serde(rename = "type")]
    pub maneuver_type: String,
}

/// Intersection information
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Intersection {
    pub out: Option<i32>,
    pub entry: Vec<bool>,
    pub bearings: Vec<i32>,
    pub location: [f64; 2],
    pub in_: Option<i32>,
    #[serde(rename = "in")]
    pub in_field: Option<i32>,
}

/// Waypoint information
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Waypoint {
    pub hint: String,
    pub distance: f64,
    pub name: String,
    pub location: [f64; 2],
}

/// Helper functions for routing model serialization/deserialization

/// Parse a JSON string into a RouteResponse struct
pub fn parse_route_json(json_str: &str) -> Result<RouteResponse, Box<dyn Error>> {
    let route_response: RouteResponse = serde_json::from_str(json_str)?;
    Ok(route_response)
}

/// Convert a RouteResponse struct to a JSON string
pub fn route_to_json(route: &RouteResponse) -> Result<String, Box<dyn Error>> {
    let json = serde_json::to_string(route)?;
    Ok(json)
}

/// Serialize a RouteResponse struct to MessagePack format
pub fn route_to_msgpack(route: &RouteResponse) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut buf = Vec::new();
    route.serialize(&mut Serializer::new(&mut buf))?;
    Ok(buf)
}

/// Deserialize MessagePack data to a RouteResponse struct
pub fn msgpack_to_route(data: &[u8]) -> Result<RouteResponse, Box<dyn Error>> {
    let mut de = Deserializer::new(Cursor::new(data));
    let route = RouteResponse::deserialize(&mut de)?;
    Ok(route)
}

/// Convert MessagePack data to JSON string
pub fn msgpack_to_json(data: &[u8]) -> Result<String, Box<dyn Error>> {
    let route = msgpack_to_route(data)?;
    route_to_json(&route)
}

/// Convert JSON string to MessagePack data
pub fn json_to_msgpack(json_str: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let route = parse_route_json(json_str)?;
    route_to_msgpack(&route)
} 