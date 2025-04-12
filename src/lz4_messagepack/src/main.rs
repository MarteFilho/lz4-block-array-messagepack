use rmpv::Value;
use rmpv::encode::write_value;
use rmpv::decode::read_value;
use std::io::{self, Read, Write, Cursor};
use std::fs::File;
use std::env;
use serde_json::{json, Value as JsonValue};
use lz4::block::decompress;

/// Represents output format options
#[derive(Debug, Clone, PartialEq)]
pub enum OutputFormat {
    Json,
    Hex,
    Binary,
    Human,
}

impl From<&str> for OutputFormat {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "hex" => OutputFormat::Hex,
            "binary" => OutputFormat::Binary,
            "human" => OutputFormat::Human,
            _ => OutputFormat::Json,
        }
    }
}

/// Represents a MessagePack extension block
#[derive(Debug)]
pub struct MessagePackExt {
    ext_type: i8,
    header_data: Vec<u8>,
    data: Vec<u8>,
}

/// Core functionality for processing LZ4 MessagePack data
pub struct LZ4MessagePackProcessor;

impl LZ4MessagePackProcessor {
    /// Parse input JSON into a MessagePackExt structure
    fn parse_input(input_json: &str) -> Result<MessagePackExt, String> {
        let parsed: JsonValue = serde_json::from_str(input_json)
            .map_err(|e| format!("Failed to parse JSON: {}", e))?;
        
        let parsed_array = parsed.as_array()
            .ok_or("Expected a JSON array")?;
        
        if parsed_array.len() < 2 {
            return Err("Input JSON must contain at least 2 elements".to_string());
        }
        
        // Extract the type and buffer from the first element
        let first_element = &parsed_array[0];
        let ext_type = first_element["type"].as_u64()
            .ok_or("Expected 'type' to be a number")?
            as i8;
        
        let header_data = Self::extract_byte_array(&first_element["buffer"]["data"])?;
        
        // Extract the data from the second element
        let data = Self::extract_byte_array(&parsed_array[1]["data"])?;
        
        Ok(MessagePackExt {
            ext_type,
            header_data,
            data,
        })
    }
    
    /// Helper function to extract a byte array from JSON
    fn extract_byte_array(json_array: &JsonValue) -> Result<Vec<u8>, String> {
        json_array.as_array()
            .ok_or("Expected data to be an array")?
            .iter()
            .map(|v| {
                v.as_u64()
                    .ok_or("Expected data element to be a number")
                    .map(|n| n as u8)
            })
            .collect::<Result<Vec<u8>, &str>>()
            .map_err(|e| e.to_string())
    }
    
    /// Calculate the uncompressed size from header data
    fn get_uncompressed_size(header: &[u8]) -> usize {
        if header.len() >= 2 {
            // Assuming the header is in the format where the second byte is the size
            header[1] as usize
        } else {
            0
        }
    }
    
    /// Reserialize the MessagePackExt back to MessagePack format
    fn reserialize_to_msgpack(ext: &MessagePackExt) -> Result<Vec<u8>, String> {
        let mut output = Vec::new();
        
        // Create a MessagePack extension object for the first part
        let ext_value = Value::Ext(ext.ext_type, ext.header_data.clone());
        
        // Create the second part with the buffer data
        let buffer_value = Value::Binary(ext.data.clone());
        
        // Create the final array
        let final_array = Value::Array(vec![ext_value, buffer_value]);
        
        // Serialize to MessagePack
        write_value(&mut output, &final_array)
            .map_err(|e| format!("Failed to serialize to MessagePack: {}", e))?;
        
        Ok(output)
    }
    
    /// Attempt to decompress data using different strategies
    fn decompress_data(data: &[u8], uncompressed_size: usize) -> Option<(Vec<u8>, usize)> {
        // Check for empty data
        if data.is_empty() {
            return Some((Vec::new(), 0));
        }
        
        // Try each decompression strategy sequentially
        
        // 1. Standard LZ4 decompression with estimated size
        match decompress(data, Some((uncompressed_size * 2) as i32)) {
            Ok(decompressed) => return Some((decompressed, 1)),
            Err(e) => eprintln!("Decompression attempt 1 failed: {}", e),
        }
        
        // 2. Try without size hint
        match decompress(data, None) {
            Ok(decompressed) => return Some((decompressed, 2)),
            Err(e) => eprintln!("Decompression attempt 2 failed: {}", e),
        }
        
        // 3. Try with offset 1
        if data.len() > 1 {
            match decompress(&data[1..], None) {
                Ok(decompressed) => return Some((decompressed, 3)),
                Err(e) => eprintln!("Decompression attempt 3 failed: {}", e),
            }
        }
        
        // 4. Try with offset 2
        if data.len() > 2 {
            match decompress(&data[2..], None) {
                Ok(decompressed) => return Some((decompressed, 4)),
                Err(e) => eprintln!("Decompression attempt 4 failed: {}", e),
            }
        }
        
        None
    }
    
    /// Process the decompressed data and convert to a more readable format
    fn process_decompressed_data(decompressed: &[u8], attempt_num: usize) -> Result<JsonValue, String> {
        eprintln!("Decompression attempt {} succeeded, got {} bytes", attempt_num, decompressed.len());
        
        // Return error for empty data
        if decompressed.is_empty() {
            return Err("Empty data after decompression".to_string());
        }
        
        // Try to parse as MessagePack
        match read_value(&mut Cursor::new(decompressed)) {
            Ok(value) => {
                eprintln!("Successfully parsed MessagePack data");
                
                // Convert to JSON
                let json_value = Self::convert_value_to_json(&value);
                
                // If the value is an array with at least 5 elements, try to extract common fields
                if let JsonValue::Array(items) = &json_value {
                    if items.len() >= 5 {
                        let mut result = serde_json::Map::new();
                        
                        // Try to extract common fields if they match the expected types
                        if let Some(JsonValue::String(type_val)) = items.get(0) {
                            result.insert("type".to_string(), json!(type_val));
                        }
                        if let Some(JsonValue::String(title)) = items.get(1) {
                            result.insert("title".to_string(), json!(title));
                        }
                        if let Some(JsonValue::Number(status)) = items.get(2) {
                            result.insert("status".to_string(), json!(status));
                        }
                        if let Some(JsonValue::String(detail)) = items.get(3) {
                            result.insert("detail".to_string(), json!(detail));
                        }
                        if let Some(JsonValue::String(instance)) = items.get(4) {
                            result.insert("instance".to_string(), json!(instance));
                        }
                        
                        // If we found any of the expected fields, return the structured object
                        if !result.is_empty() {
                            return Ok(JsonValue::Object(result));
                        }
                    }
                }
                
                // Otherwise return the raw converted value
                Ok(json_value)
            },
            Err(e) => {
                eprintln!("Failed to parse decompressed data as MessagePack: {}", e);
                
                // Try to interpret as UTF-8 string
                match String::from_utf8(decompressed.to_vec()) {
                    Ok(s) => {
                        if s.chars().any(|c| !c.is_control()) {
                            eprintln!("Interpreted as UTF-8 string");
                            Ok(json!({ "raw_string": s }))
                        } else {
                            Err("Could not parse data as MessagePack or UTF-8 string".to_string())
                        }
                    },
                    Err(_) => Err("Could not parse data as MessagePack or UTF-8 string".to_string()),
                }
            }
        }
    }
    
    /// Convert a MessagePack value to a JSON value
    fn convert_value_to_json(value: &Value) -> JsonValue {
        match value {
            Value::Nil => JsonValue::Null,
            Value::Boolean(b) => json!(*b),
            Value::Integer(i) => json!(i.as_i64()),
            Value::F32(f) => json!(*f),
            Value::F64(f) => json!(*f),
            Value::String(s) => {
                if let Some(text) = s.as_str() {
                    json!(text)
                } else {
                    json!(null)
                }
            },
            Value::Binary(b) => json!(b.iter().map(|&byte| byte).collect::<Vec<u8>>()),
            Value::Array(a) => {
                json!(a.iter().map(Self::convert_value_to_json).collect::<Vec<_>>())
            },
            Value::Map(m) => {
                let mut obj = serde_json::Map::new();
                for (k, v) in m {
                    if let Value::String(key_str) = k {
                        if let Some(key) = key_str.as_str() {
                            obj.insert(key.to_string(), Self::convert_value_to_json(v));
                        }
                    }
                }
                JsonValue::Object(obj)
            },
            Value::Ext(typ, data) => {
                json!({
                    "ext_type": typ,
                    "ext_data": data.iter().map(|&byte| byte).collect::<Vec<u8>>()
                })
            }
        }
    }
    
    /// Process an input file or string and output the result
    pub fn process(input_source: Option<&str>, output_format: OutputFormat) -> Result<String, String> {
        // Read input JSON
        let input_json = Self::read_input(input_source)?;
        
        // Parse the input
        let ext = Self::parse_input(&input_json)?;
        
        eprintln!("Ext type: {}", ext.ext_type);
        eprintln!("Header data length: {}", ext.header_data.len());
        eprintln!("Compressed data length: {}", ext.data.len());
        
        // Process based on the extension type
        if ext.ext_type == 98 { // LZ4BlockArray
            // Get expected uncompressed size
            let uncompressed_size = Self::get_uncompressed_size(&ext.header_data);
            eprintln!("Expected uncompressed size from header: {}", uncompressed_size);
            
            // Reserialize to MessagePack
            let msgpack_output = Self::reserialize_to_msgpack(&ext)?;
            eprintln!("MessagePack output length: {} bytes", msgpack_output.len());
            
            // Try to decompress
            let human_readable = match Self::decompress_data(&ext.data, uncompressed_size) {
                Some((decompressed, attempt)) => Self::process_decompressed_data(&decompressed, attempt)
                    .unwrap_or_else(|_| json!({ "error": "Failed to process decompressed data" })),
                None => json!({ "error": "Failed to decompress data after multiple attempts" }),
            };
            
            // Generate output based on format
            match output_format {
                OutputFormat::Binary => {
                    // For binary output, just return the raw bytes
                    // This isn't ideal for a String result, but the caller can handle it
                    return Ok("Binary data generated, use stdout for binary output".to_string());
                },
                OutputFormat::Hex => {
                    // Return hex representation
                    Ok(msgpack_output.iter().map(|b| format!("{:02x}", b)).collect())
                },
                OutputFormat::Human => {
                    // Return human-readable JSON
                    Ok(serde_json::to_string_pretty(&human_readable)
                        .map_err(|e| format!("Error formatting JSON: {}", e))?)
                },
                OutputFormat::Json => {
                    // Return full JSON with all details
                    let result = json!({
                        "messagepack_hex": msgpack_output.iter().map(|b| format!("{:02x}", b)).collect::<String>(),
                        "messagepack_length": msgpack_output.len(),
                        "original_ext_type": ext.ext_type,
                        "original_header_data": ext.header_data.iter().map(|b| format!("{:02x}", b)).collect::<String>(),
                        "original_data_length": ext.data.len(),
                        "human_readable": human_readable
                    });
                    
                    Ok(serde_json::to_string_pretty(&result)
                        .map_err(|e| format!("Error formatting JSON: {}", e))?)
                }
            }
        } else {
            Err(format!("Unsupported extension type: {}", ext.ext_type))
        }
    }
    
    /// Read input from a file, stdin, or use default data
    fn read_input(source: Option<&str>) -> Result<String, String> {
        match source {
            Some("-") => {
                // Read from stdin
                let mut buffer = String::new();
                io::stdin().read_to_string(&mut buffer)
                    .map_err(|e| format!("Failed to read from stdin: {}", e))?;
                Ok(buffer)
            },
            Some(path) => {
                // Read from file
                let mut file = File::open(path)
                    .map_err(|e| format!("Failed to open file {}: {}", path, e))?;
                let mut buffer = String::new();
                file.read_to_string(&mut buffer)
                    .map_err(|e| format!("Failed to read file {}: {}", path, e))?;
                Ok(buffer)
            },
            None => {
                // Use default test data
                eprintln!("No input file specified, using default test data.");
                Ok(include_str!("../default_input.json").to_string())
            }
        }
    }
    
    /// Output binary data to stdout
    pub fn write_binary_to_stdout(data: &[u8]) -> Result<(), String> {
        io::stdout().write_all(data)
            .map_err(|e| format!("Failed to write binary data: {}", e))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    
    // Show usage if --help or -h is provided
    if args.len() > 1 && (args[1] == "--help" || args[1] == "-h") {
        println!("Usage: {} [INPUT_FILE|-] [FORMAT]", args[0]);
        println!("Formats: json (default), hex, binary, human");
        println!("  json   - Output detailed JSON with all metadata");
        println!("  hex    - Output just the hex representation of MessagePack data");
        println!("  binary - Output raw binary MessagePack data");
        println!("  human  - Output human-readable interpretation of the data");
        println!("\nExamples:");
        println!("  {} input.json            # Process file with JSON output", args[0]);
        println!("  {} input.json human      # Process file with human-readable output", args[0]);
        println!("  {} human                 # Process default data with human-readable output", args[0]);
        println!("  cat input.json | {} -    # Process stdin with JSON output", args[0]);
        return Ok(());
    }
    
    // Parse input file and output format
    let (input_file, output_format) = if args.len() > 1 && ["human", "hex", "binary", "json"].contains(&args[1].as_str()) {
        (None, OutputFormat::from(args[1].as_str()))
    } else if args.len() > 2 {
        (Some(args[1].as_str()), OutputFormat::from(args[2].as_str()))
    } else if args.len() > 1 {
        (Some(args[1].as_str()), OutputFormat::Json)
    } else {
        (None, OutputFormat::Json)
    };
    
    // Process the input
    let result = LZ4MessagePackProcessor::process(input_file, output_format.clone())?;
    
    // Handle special case for binary output
    if output_format == OutputFormat::Binary {
        // For binary output, we need to reprocess to get the actual bytes
        let ext = LZ4MessagePackProcessor::parse_input(
            &LZ4MessagePackProcessor::read_input(input_file)?
        )?;
        
        let msgpack_output = LZ4MessagePackProcessor::reserialize_to_msgpack(&ext)?;
        LZ4MessagePackProcessor::write_binary_to_stdout(&msgpack_output)?;
    } else {
        // For text-based outputs, just print the result
        println!("{}", result);
    }
    
    Ok(())
} 