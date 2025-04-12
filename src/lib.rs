use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use serde_json::Value as JsonValue;
use rmpv::Value;
use rmpv::encode::write_value;
use lz4::block::compress;
use std::io::Cursor;

#[no_mangle]
pub extern "C" fn process_lz4_messagepack(input_json: *const c_char) -> *mut c_char {
    let input_str = unsafe {
        if input_json.is_null() {
            return CString::new("Error: Null input").unwrap().into_raw();
        }
        match CStr::from_ptr(input_json).to_str() {
            Ok(s) => s,
            Err(_) => return CString::new("Error: Invalid UTF-8").unwrap().into_raw(),
        }
    };

    let result = process_json(input_str);
    match result {
        Ok(output) => CString::new(output).unwrap().into_raw(),
        Err(e) => CString::new(format!("Error: {}", e)).unwrap().into_raw(),
    }
}

fn process_json(input: &str) -> Result<String, String> {
    // Parse input JSON
    let json_value: JsonValue = serde_json::from_str(input)
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;

    // Convert JSON to MessagePack Value
    let msgpack_value = convert_json_to_msgpack(&json_value)?;

    // Serialize to MessagePack
    let mut buffer = Vec::new();
    write_value(&mut buffer, &msgpack_value)
        .map_err(|e| format!("Failed to serialize MessagePack: {}", e))?;

    // Compress with LZ4
    let compressed_data = compress(&buffer, None, false)
        .map_err(|e| format!("Failed to compress with LZ4: {}", e))?;

    // Create output JSON structure
    let output_json = create_output_json(&buffer, &compressed_data)?;

    // Serialize to JSON string
    serde_json::to_string_pretty(&output_json)
        .map_err(|e| format!("Failed to serialize output JSON: {}", e))
}

fn convert_json_to_msgpack(json: &JsonValue) -> Result<Value, String> {
    match json {
        JsonValue::Null => Ok(Value::Nil),
        JsonValue::Bool(b) => Ok(Value::Boolean(*b)),
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(Value::Integer(i.into()))
            } else if let Some(f) = n.as_f64() {
                Ok(Value::F64(f))
            } else {
                Err("Invalid number".to_string())
            }
        }
        JsonValue::String(s) => Ok(Value::String(s.into())),
        JsonValue::Array(arr) => {
            let mut result = Vec::new();
            for item in arr {
                result.push(convert_json_to_msgpack(item)?);
            }
            Ok(Value::Array(result))
        }
        JsonValue::Object(obj) => {
            let mut result = Vec::new();
            for (key, value) in obj {
                result.push((Value::String(key.into()), convert_json_to_msgpack(value)?));
            }
            Ok(Value::Map(result))
        }
    }
}

fn create_output_json(uncompressed: &[u8], compressed: &[u8]) -> Result<JsonValue, String> {
    // Create header data
    let mut header_data = Vec::new();
    header_data.push(204); // Type byte
    
    // Encode size in big-endian
    let size = uncompressed.len();
    if size <= 0xFF {
        header_data.push(size as u8);
    } else if size <= 0xFFFF {
        header_data.push((size >> 8) as u8);
        header_data.push(size as u8);
    } else if size <= 0xFFFFFF {
        header_data.push((size >> 16) as u8);
        header_data.push((size >> 8) as u8);
        header_data.push(size as u8);
    } else {
        header_data.push((size >> 24) as u8);
        header_data.push((size >> 16) as u8);
        header_data.push((size >> 8) as u8);
        header_data.push(size as u8);
    }

    Ok(json!([
        {
            "buffer": {
                "type": "Buffer",
                "data": header_data
            },
            "type": 98
        },
        {
            "type": "Buffer",
            "data": compressed
        }
    ]))
}

#[no_mangle]
pub extern "C" fn free_string(ptr: *mut c_char) {
    unsafe {
        if !ptr.is_null() {
            let _ = CString::from_raw(ptr);
        }
    }
} 