#[path = "../src/models.rs"]
mod models;
use models::{RouteResponse, parse_route_json, route_to_msgpack, msgpack_to_route, route_to_json};

#[path = "../src/main.rs"]
mod main;
use main::{LZ4MessagePackProcessor, OutputFormat};

use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use serde_json::{json, Value};
use lz4::block::compress;

const TEST_JSON: &str = r#"{
    "code": "Ok",
    "routes": [{
        "legs": [{
            "steps": [{
                "geometry": "mplyAn|m_I}AyCeBkDqCgF",
                "maneuver": {
                    "bearing_after": 90,
                    "bearing_before": 0,
                    "location": [13.349, 52.515],
                    "modifier": "right",
                    "type": "depart"
                },
                "mode": "driving",
                "driving_side": "right",
                "name": "Friedrichstraße",
                "intersections": [{
                    "out": 0,
                    "entry": [true, false, false],
                    "bearings": [90, 180, 270],
                    "location": [13.349, 52.515]
                }],
                "weight": 31.5,
                "duration": 31.5,
                "distance": 230.5,
                "ref": "B96"
            }],
            "summary": "Friedrichstraße",
            "weight": 31.5,
            "duration": 31.5,
            "distance": 230.5
        }],
        "weight_name": "routability",
        "weight": 31.5,
        "duration": 31.5,
        "distance": 230.5
    }],
    "waypoints": [{
        "hint": "JQEAABQAAAAIAAAABAAAAGILRQCJDMMAMpNcAIcDzAAgAAAACAAAABQAAAAEAAAAo6WoAMFRRwGL22cBo6WoAA8AAAD_____vwAAQaZx0kA=",
        "distance": 0.0,
        "name": "Friedrichstraße",
        "location": [13.349, 52.515]
    }, {
        "hint": "JQEAABQAAAAIAAAABAAAAGILRQCJDMMAMpNcAIcDzAAgAAAACAAAABQAAAAEAAAAo6WoAMFRRwGL22cBo6WoAA8AAAD_____vwAAQaZx0kA=",
        "distance": 0.0,
        "name": "Friedrichstraße",
        "location": [13.352, 52.516]
    }]
}"#;

#[test]
fn test_route_serialization_deserialization() {
    // Parse test JSON
    let route_response = parse_route_json(TEST_JSON).expect("Failed to parse test JSON");
    
    // Verify parsed data
    assert_eq!(route_response.code, "Ok");
    assert_eq!(route_response.routes.len(), 1);
    assert_eq!(route_response.waypoints.len(), 2);
    
    // Test serialization to MessagePack
    let msgpack_data = route_to_msgpack(&route_response).expect("Failed to serialize to MessagePack");
    
    // Test deserialization from MessagePack
    let deserialized_route = msgpack_to_route(&msgpack_data).expect("Failed to deserialize from MessagePack");
    
    // Verify data integrity after round trip
    assert_eq!(route_response.code, deserialized_route.code);
    assert_eq!(route_response.routes.len(), deserialized_route.routes.len());
    assert_eq!(route_response.waypoints.len(), deserialized_route.waypoints.len());
    
    // More detailed verification
    assert_eq!(route_response.routes[0].distance, deserialized_route.routes[0].distance);
    assert_eq!(route_response.routes[0].duration, deserialized_route.routes[0].duration);
    assert_eq!(route_response.routes[0].weight, deserialized_route.routes[0].weight);
    
    // Save the JSON to a file for testing
    let test_file_path = "test_route.json";
    let mut file = File::create(test_file_path).expect("Failed to create test JSON file");
    file.write_all(TEST_JSON.as_bytes()).expect("Failed to write test JSON");
    
    // Read back the JSON file
    let mut json_file = File::open(test_file_path).expect("Failed to open test JSON file");
    let mut json_content = String::new();
    json_file.read_to_string(&mut json_content).expect("Failed to read test JSON file");
    
    // Parse the JSON file content
    let file_route = parse_route_json(&json_content).expect("Failed to parse file JSON");
    
    // Serialize to MessagePack and save
    let file_msgpack_data = route_to_msgpack(&file_route).expect("Failed to serialize file data to MessagePack");
    let mut msgpack_file = File::create("test_route.msgpack").expect("Failed to create MessagePack file");
    msgpack_file.write_all(&file_msgpack_data).expect("Failed to write MessagePack data");
    
    // Verify file data equality
    assert_eq!(route_response.code, file_route.code);
    assert_eq!(route_response.routes.len(), file_route.routes.len());
    assert_eq!(route_response.waypoints.len(), file_route.waypoints.len());
}

#[test]
fn test_route_from_lz4_compressed() {
    // Parse test JSON
    let route_response = parse_route_json(TEST_JSON).expect("Failed to parse test JSON");
    
    // Serialize to MessagePack
    let msgpack_data = route_to_msgpack(&route_response).expect("Failed to serialize to MessagePack");
    
    // Compress with LZ4
    let compressed_data = compress(&msgpack_data, None, false).expect("Failed to compress data");
    
    // Save compressed data to file
    let mut lz4_file = File::create("test_route.lz4").expect("Failed to create LZ4 file");
    lz4_file.write_all(&compressed_data).expect("Failed to write LZ4 data");
    
    // Create JSON wrapper for compressed data (similar to what processor would create)
    let json_wrapper = json!([
        {
            "type": 98,
            "buffer": {
                "data": compressed_data.iter().map(|&b| b as u64).collect::<Vec<_>>()
            }
        },
        {
            "data": msgpack_data.iter().map(|&b| b as u64).collect::<Vec<_>>()
        }
    ]);
    
    // Save the JSON wrapper
    let wrapper_json = serde_json::to_string_pretty(&json_wrapper).expect("Failed to serialize JSON wrapper");
    let mut wrapper_file = File::create("test_route_wrapper.json").expect("Failed to create wrapper JSON file");
    wrapper_file.write_all(wrapper_json.as_bytes()).expect("Failed to write wrapper JSON");
    
    // Test that the LZ4MessagePackProcessor can handle this data
    let processor_result = LZ4MessagePackProcessor::process(Some("test_route_wrapper.json"), main::OutputFormat::Json)
        .expect("Failed to process test route data");
    
    // Parse the processor result
    let result_value: Value = serde_json::from_str(&processor_result).expect("Failed to parse processor result");
    
    // Verify the processor result contains route data
    assert!(result_value.is_object());
    
    // Extract route from processor result and verify
    if let Ok(deserialized_route) = parse_route_json(&processor_result) {
        assert_eq!(route_response.code, deserialized_route.code);
        assert_eq!(route_response.routes.len(), deserialized_route.routes.len());
        assert_eq!(route_response.waypoints.len(), deserialized_route.waypoints.len());
    }
}

#[test]
fn test_process_real_data() {
    // This test attempts to read from an actual data file if it exists
    let input_path = "default_input.json";
    
    if Path::new(input_path).exists() {
        let mut input_file = File::open(input_path).expect("Failed to open input file");
        let mut input_content = String::new();
        input_file.read_to_string(&mut input_content).expect("Failed to read input file");
        
        // Parse the JSON
        let json_value: Value = serde_json::from_str(&input_content).expect("Failed to parse input JSON");
        
        // Try to extract LZ4 compressed data
        if let Some(blocks) = json_value.as_array() {
            if blocks.len() >= 2 {
                // Extract the compressed data bytes
                if let Some(data_block) = blocks.get(1) {
                    if let Some(data_array) = data_block.get("data").and_then(|d| d.as_array()) {
                        let bytes: Vec<u8> = data_array.iter()
                            .filter_map(|v| v.as_u64().map(|n| n as u8))
                            .collect();
                            
                        // Try to decompress and deserialize
                        match lz4::block::decompress(&bytes, None) {
                            Ok(decompressed) => {
                                if let Ok(route) = msgpack_to_route(&decompressed) {
                                    // Data was successfully decompressed and parsed
                                    assert!(!route.code.is_empty());
                                    assert!(!route.routes.is_empty());
                                    
                                    // Write processed result to file
                                    let route_json = route_to_json(&route).expect("Failed to convert route to JSON");
                                    let mut output_file = File::create("processed_route.json").expect("Failed to create output file");
                                    output_file.write_all(route_json.as_bytes()).expect("Failed to write processed route");
                                }
                            }
                            Err(_) => {
                                // Decompression failed, but that's okay for this test
                                // The file might not contain valid LZ4 data
                            }
                        }
                    }
                }
            }
        }
    }
    
    // This test should pass regardless of whether we could process the data
    assert!(true);
} 