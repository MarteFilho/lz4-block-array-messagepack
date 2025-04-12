use serde_json::{json, Value as JsonValue};
use std::fs;
use std::path::Path;
use lz4::block::compress;
use rmpv::Value;
use rmpv::encode::write_value;
use std::io::Cursor;

// Import o código da aplicação principal
#[path = "../src/main.rs"]
mod app;
use app::LZ4MessagePackProcessor;
use app::OutputFormat;

// Função auxiliar para comprimir dados e criar o JSON de teste
fn create_test_data(value: &Value) -> JsonValue {
    // Serializar o valor para MessagePack
    let mut buffer = Vec::new();
    write_value(&mut buffer, value).unwrap();
    
    // Comprimir os dados com LZ4
    let compressed_data = compress(&buffer, None, false).unwrap_or_default();
    
    // Criar o buffer de cabeçalho
    let mut header_data = Vec::new();
    header_data.push(204); // Tipo fixo
    
    // Codificar o tamanho descomprimido em big-endian
    let size = buffer.len();
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
    
    // Criar o JSON com a estrutura LZ4BlockArray
    json!([
        {
            "buffer": {
                "type": "Buffer",
                "data": header_data
            },
            "type": 98
        },
        {
            "type": "Buffer",
            "data": compressed_data
        }
    ])
}

// Função auxiliar para gerar arquivos de teste
fn generate_test_file(name: &str, content: &JsonValue) -> String {
    let test_dir = Path::new("tests/examples");
    if !test_dir.exists() {
        fs::create_dir_all(test_dir).expect("Failed to create test directory");
    }
    
    let file_path = test_dir.join(format!("{}.json", name));
    let json_content = serde_json::to_string_pretty(content).expect("Failed to serialize JSON");
    
    fs::write(&file_path, json_content).expect("Failed to write test file");
    file_path.to_string_lossy().to_string()
}

#[test]
fn test_simple_array() {
    let value = Value::Array(vec![
        Value::Integer(1.into()),
        Value::Integer(2.into()),
        Value::Integer(3.into()),
        Value::Integer(4.into()),
        Value::Integer(5.into())
    ]);
    
    let test_data = create_test_data(&value);
    let file_path = generate_test_file("simple_array", &test_data);
    let result = LZ4MessagePackProcessor::process(Some(&file_path), OutputFormat::Human);
    assert!(result.is_ok());
}

#[test]
fn test_nested_objects() {
    let value = Value::Map(vec![
        (Value::String("name".into()), Value::String("Product".into())),
        (Value::String("price".into()), Value::F64(29.99)),
        (Value::String("in_stock".into()), Value::Boolean(true)),
        (Value::String("tags".into()), Value::Array(vec![
            Value::String("electronics".into()),
            Value::String("gadget".into())
        ])),
        (Value::String("details".into()), Value::Map(vec![
            (Value::String("manufacturer".into()), Value::String("Example Corp".into())),
            (Value::String("model".into()), Value::String("X123".into())),
            (Value::String("dimensions".into()), Value::Map(vec![
                (Value::String("width".into()), Value::Integer(10.into())),
                (Value::String("height".into()), Value::Integer(5.into())),
                (Value::String("depth".into()), Value::Integer(2.into()))
            ]))
        ]))
    ]);
    
    let test_data = create_test_data(&value);
    let file_path = generate_test_file("nested_objects", &test_data);
    let result = LZ4MessagePackProcessor::process(Some(&file_path), OutputFormat::Human);
    assert!(result.is_ok());
}

#[test]
fn test_mixed_types() {
    let value = Value::Array(vec![
        Value::String("string value".into()),
        Value::Integer(42.into()),
        Value::Boolean(true),
        Value::Nil,
        Value::Array(vec![
            Value::Integer(1.into()),
            Value::Integer(2.into()),
            Value::Integer(3.into())
        ]),
        Value::Map(vec![
            (Value::String("key".into()), Value::String("value".into()))
        ])
    ]);
    
    let test_data = create_test_data(&value);
    let file_path = generate_test_file("mixed_types", &test_data);
    let result = LZ4MessagePackProcessor::process(Some(&file_path), OutputFormat::Human);
    assert!(result.is_ok());
}

#[test]
fn test_special_characters() {
    let value = Value::Array(vec![
        Value::String("Caracteres especiais: áéíóú àèìòù âêîôû ãõ ç ñ".into()),
        Value::String("特殊文字: 漢字 カタカナ ひらがな".into()),
        Value::String("Emoji: 😀 🎉 🌍 🚀".into()),
        Value::String("Symbols: © ® ™ € ¥ £".into())
    ]);
    
    let test_data = create_test_data(&value);
    let file_path = generate_test_file("special_chars", &test_data);
    let result = LZ4MessagePackProcessor::process(Some(&file_path), OutputFormat::Human);
    assert!(result.is_ok());
}

#[test]
fn test_large_numbers() {
    let value = Value::Array(vec![
        Value::Integer(1234567890123456789i64.into()),
        Value::Integer((-987654321098765432i64).into()),
        Value::F64(3.141592653589793),
        Value::F64(-0.000000000000001)
    ]);
    
    let test_data = create_test_data(&value);
    let file_path = generate_test_file("large_numbers", &test_data);
    let result = LZ4MessagePackProcessor::process(Some(&file_path), OutputFormat::Human);
    assert!(result.is_ok());
}

#[test]
fn test_complex_validation() {
    let value = Value::Array(vec![
        Value::String("https://api.example.com/errors/validation".into()),
        Value::String("Validation Error".into()),
        Value::Integer(400.into()),
        Value::Map(vec![
            (Value::String("errors".into()), Value::Array(vec![
                Value::Map(vec![
                    (Value::String("field".into()), Value::String("username".into())),
                    (Value::String("message".into()), Value::String("Username is required".into()))
                ]),
                Value::Map(vec![
                    (Value::String("field".into()), Value::String("email".into())),
                    (Value::String("message".into()), Value::String("Invalid email format".into()))
                ]),
                Value::Map(vec![
                    (Value::String("field".into()), Value::String("password".into())),
                    (Value::String("message".into()), Value::String("Password must be at least 8 characters".into()))
                ])
            ]))
        ]),
        Value::String("/api/v1/users".into())
    ]);
    
    let test_data = create_test_data(&value);
    let file_path = generate_test_file("complex_validation", &test_data);
    let result = LZ4MessagePackProcessor::process(Some(&file_path), OutputFormat::Human);
    assert!(result.is_ok());
}

#[test]
fn test_binary_data() {
    let binary_data = vec![
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
        0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F,
        0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17,
        0x18, 0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E, 0x1F
    ];
    let value = Value::Binary(binary_data);
    
    let test_data = create_test_data(&value);
    let file_path = generate_test_file("binary_data", &test_data);
    let result = LZ4MessagePackProcessor::process(Some(&file_path), OutputFormat::Human);
    assert!(result.is_ok());
}

#[test]
fn test_extended_types() {
    let value = Value::Map(vec![
        (Value::String("timestamp".into()), Value::Integer(1672531200000i64.into())),
        (Value::String("uuid".into()), Value::String("550e8400-e29b-41d4-a716-446655440000".into())),
        (Value::String("binary".into()), Value::Binary(vec![1, 0, 1, 0, 1])),
        (Value::String("array".into()), Value::Array(vec![
            Value::Integer(1.into()),
            Value::String("two".into()),
            Value::Boolean(true),
            Value::Nil
        ])),
        (Value::String("nested".into()), Value::Map(vec![
            (Value::String("level1".into()), Value::Map(vec![
                (Value::String("level2".into()), Value::Map(vec![
                    (Value::String("level3".into()), Value::String("deep".into()))
                ]))
            ]))
        ]))
    ]);
    
    let test_data = create_test_data(&value);
    let file_path = generate_test_file("extended_types", &test_data);
    let result = LZ4MessagePackProcessor::process(Some(&file_path), OutputFormat::Human);
    assert!(result.is_ok());
}

#[test]
fn test_complex_real_world_data() {
    let value = Value::Array(vec![
        Value::Map(vec![
            (Value::String("full name".into()), Value::String("Felix Hills".into())),
            (Value::String("address".into()), Value::Map(vec![
                (Value::String("street".into()), Value::String("87304 Pfeffer Walk".into())),
                (Value::String("city".into()), Value::String("Kilmacanoge".into())),
                (Value::String("coordinates".into()), Value::Map(vec![
                    (Value::String("latitude".into()), Value::F64(53.1824)),
                    (Value::String("longitude".into()), Value::F64(-6.1334))
                ]))
            ])),
            (Value::String("contact".into()), Value::Map(vec![
                (Value::String("email".into()), Value::String("felix.hills@example.com".into())),
                (Value::String("phone".into()), Value::String("+1-555-123-4567".into())),
                (Value::String("social".into()), Value::Map(vec![
                    (Value::String("twitter".into()), Value::String("@felixhills".into())),
                    (Value::String("linkedin".into()), Value::String("linkedin.com/in/felixhills".into()))
                ]))
            ])),
            (Value::String("employment".into()), Value::Map(vec![
                (Value::String("company".into()), Value::String("TechCorp Inc.".into())),
                (Value::String("position".into()), Value::String("Senior Software Engineer".into())),
                (Value::String("department".into()), Value::String("Research & Development".into())),
                (Value::String("salary".into()), Value::Integer(125000.into())),
                (Value::String("start_date".into()), Value::String("2020-03-15".into())),
                (Value::String("benefits".into()), Value::Array(vec![
                    Value::String("Health Insurance".into()),
                    Value::String("401k".into()),
                    Value::String("Stock Options".into())
                ]))
            ])),
            (Value::String("education".into()), Value::Array(vec![
                Value::Map(vec![
                    (Value::String("institution".into()), Value::String("MIT".into())),
                    (Value::String("degree".into()), Value::String("PhD in Computer Science".into())),
                    (Value::String("graduation_year".into()), Value::Integer(2018.into())),
                    (Value::String("gpa".into()), Value::F64(3.9))
                ]),
                Value::Map(vec![
                    (Value::String("institution".into()), Value::String("Stanford University".into())),
                    (Value::String("degree".into()), Value::String("MSc in Artificial Intelligence".into())),
                    (Value::String("graduation_year".into()), Value::Integer(2015.into())),
                    (Value::String("gpa".into()), Value::F64(3.8))
                ])
            ])),
            (Value::String("skills".into()), Value::Map(vec![
                (Value::String("programming".into()), Value::Array(vec![
                    Value::String("Rust".into()),
                    Value::String("Python".into()),
                    Value::String("Go".into()),
                    Value::String("JavaScript".into())
                ])),
                (Value::String("databases".into()), Value::Array(vec![
                    Value::String("PostgreSQL".into()),
                    Value::String("MongoDB".into()),
                    Value::String("Redis".into())
                ])),
                (Value::String("cloud".into()), Value::Array(vec![
                    Value::String("AWS".into()),
                    Value::String("GCP".into()),
                    Value::String("Azure".into())
                ]))
            ])),
            (Value::String("projects".into()), Value::Array(vec![
                Value::Map(vec![
                    (Value::String("name".into()), Value::String("Distributed Systems Framework".into())),
                    (Value::String("description".into()), Value::String("A high-performance distributed computing framework".into())),
                    (Value::String("technologies".into()), Value::Array(vec![
                        Value::String("Rust".into()),
                        Value::String("gRPC".into()),
                        Value::String("Kubernetes".into())
                    ])),
                    (Value::String("status".into()), Value::String("Active".into())),
                    (Value::String("start_date".into()), Value::String("2021-01-01".into())),
                    (Value::String("team_size".into()), Value::Integer(5.into()))
                ]),
                Value::Map(vec![
                    (Value::String("name".into()), Value::String("AI Model Training Pipeline".into())),
                    (Value::String("description".into()), Value::String("Automated ML model training and deployment system".into())),
                    (Value::String("technologies".into()), Value::Array(vec![
                        Value::String("Python".into()),
                        Value::String("TensorFlow".into()),
                        Value::String("Docker".into())
                    ])),
                    (Value::String("status".into()), Value::String("Completed".into())),
                    (Value::String("start_date".into()), Value::String("2020-06-15".into())),
                    (Value::String("end_date".into()), Value::String("2021-03-20".into())),
                    (Value::String("team_size".into()), Value::Integer(3.into()))
                ])
            ])),
            (Value::String("preferences".into()), Value::Map(vec![
                (Value::String("theme".into()), Value::String("dark".into())),
                (Value::String("language".into()), Value::String("en-US".into())),
                (Value::String("timezone".into()), Value::String("UTC+1".into())),
                (Value::String("notifications".into()), Value::Map(vec![
                    (Value::String("email".into()), Value::Boolean(true)),
                    (Value::String("push".into()), Value::Boolean(true)),
                    (Value::String("sms".into()), Value::Boolean(false))
                ]))
            ]))
        ])
    ]);
    
    let test_data = create_test_data(&value);
    let file_path = generate_test_file("complex_real_world", &test_data);
    let result = LZ4MessagePackProcessor::process(Some(&file_path), OutputFormat::Human);
    assert!(result.is_ok());
} 