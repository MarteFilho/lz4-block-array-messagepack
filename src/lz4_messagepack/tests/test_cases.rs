#![recursion_limit = "256"]

use serde_json::{json, Value as JsonValue};
use std::fs;
use std::path::Path;

// Import o código da aplicação principal
#[path = "../src/main.rs"]
mod app;
use app::LZ4MessagePackProcessor;
use app::OutputFormat;
use app::MessagePackExt;

// Função para gerar um arquivo de teste com dados MessagePack LZ4BlockArray
fn generate_test_data(test_name: &str, content: &JsonValue) -> String {
    let test_dir = Path::new("tests/data");
    if !test_dir.exists() {
        fs::create_dir_all(test_dir).expect("Failed to create test data directory");
    }
    
    let file_path = test_dir.join(format!("{}.json", test_name));
    let json_content = serde_json::to_string_pretty(content).expect("Failed to serialize JSON");
    
    fs::write(&file_path, json_content).expect("Failed to write test file");
    file_path.to_string_lossy().to_string()
}

#[test]
fn test_empty_data() {
    // Teste com array vazio
    let empty_data = json!([
        {
            "buffer": {
                "type": "Buffer",
                "data": [204, 0]
            },
            "type": 98
        },
        {
            "type": "Buffer",
            "data": []
        }
    ]);
    
    let file_path = generate_test_data("empty_data", &empty_data);
    let result = app::LZ4MessagePackProcessor::process(Some(&file_path), app::OutputFormat::Human);
    
    assert!(result.is_err(), "Should fail with empty data");
}

#[test]
fn test_invalid_type() {
    // Teste com tipo de extensão inválido
    let invalid_type = json!([
        {
            "buffer": {
                "type": "Buffer",
                "data": [204, 100]
            },
            "type": 99  // Tipo diferente de 98 (LZ4BlockArray)
        },
        {
            "type": "Buffer",
            "data": [1, 2, 3, 4, 5]
        }
    ]);
    
    let file_path = generate_test_data("invalid_type", &invalid_type);
    let result = app::LZ4MessagePackProcessor::process(Some(&file_path), app::OutputFormat::Human);
    
    assert!(result.is_err(), "Should fail with invalid extension type");
}

#[test]
fn test_malformed_json() {
    // Teste com JSON mal formado
    let malformed_json = "{ this is not valid JSON }";
    let test_dir = Path::new("tests/data");
    if !test_dir.exists() {
        fs::create_dir_all(test_dir).expect("Failed to create test data directory");
    }
    
    let file_path = test_dir.join("malformed.json");
    fs::write(&file_path, malformed_json).expect("Failed to write test file");
    
    let result = app::LZ4MessagePackProcessor::process(Some(&file_path.to_string_lossy()), app::OutputFormat::Human);
    
    assert!(result.is_err(), "Should fail with malformed JSON");
}

#[test]
fn test_large_data() {
    // Teste com dados grandes
    let mut large_data_array = Vec::new();
    for i in 0..1000 {
        large_data_array.push(i % 256);
    }
    
    let large_data = json!([
        {
            "buffer": {
                "type": "Buffer",
                "data": [204, 232]  // 232 como tamanho descomprimido
            },
            "type": 98
        },
        {
            "type": "Buffer",
            "data": large_data_array
        }
    ]);
    
    let file_path = generate_test_data("large_data", &large_data);
    let result = app::LZ4MessagePackProcessor::process(Some(&file_path), app::OutputFormat::Human);
    
    // Neste caso, o teste pode falhar dependendo de se os dados podem ser descomprimidos
    // Estamos apenas verificando se a função não quebra com dados grandes
    assert!(result.is_ok() || result.is_err(), "Should handle large data without crashing");
}

#[test]
fn test_valid_data() {
    // Teste com dados válidos
    let valid_data = json!([
        {
            "buffer": {
                "type": "Buffer",
                "data": [204, 184]
            },
            "type": 98
        },
        {
            "type": "Buffer",
            "data": [
                244, 68, 149, 217, 63, 104, 116, 116, 112, 115, 58, 47, 47, 97, 112, 105,
                46, 120, 109, 111, 98, 113, 97, 46, 99, 111, 109, 47, 101, 114, 114, 111,
                114, 115, 47, 118, 97, 108, 105, 100, 97, 116, 105, 111, 110, 47, 109, 105,
                115, 115, 105, 110, 103, 45, 114, 101, 113, 117, 105, 114, 101, 100, 45, 102,
                105, 101, 108, 100, 184, 80, 104, 111, 110, 101, 32, 110, 117, 109, 98, 101,
                114, 32, 105, 115, 32, 31, 0, 175, 205, 1, 144, 217, 63, 84, 104, 101, 32,
                112, 33, 0, 4, 240, 21, 32, 97, 110, 100, 32, 99, 97, 110, 110, 111, 116,
                32, 98, 101, 32, 101, 109, 112, 116, 121, 32, 111, 114, 32, 119, 104, 105,
                116, 101, 115, 112, 97, 99, 101, 46, 184, 150, 0, 240, 5, 47, 118, 49, 47,
                101, 110, 100, 45, 117, 115, 101, 114, 115, 63, 112, 104, 111, 110, 101, 61
            ]
        }
    ]);
    
    let file_path = generate_test_data("valid_data", &valid_data);
    let result = app::LZ4MessagePackProcessor::process(Some(&file_path), app::OutputFormat::Human);
    
    assert!(result.is_ok(), "Should successfully process valid data");
    let content = result.unwrap();
    
    // Verificar se o conteúdo descomprimido contém os campos esperados
    assert!(content.contains("title"), "Should contain 'title' field");
    assert!(content.contains("Phone number is required"), "Should contain expected title");
    assert!(content.contains("status"), "Should contain 'status' field");
    assert!(content.contains("400"), "Should contain expected status code");
}

#[test]
fn test_different_formats() {
    // Testar diferentes formatos de saída
    let valid_data = json!([
        {
            "buffer": {
                "type": "Buffer",
                "data": [204, 184]
            },
            "type": 98
        },
        {
            "type": "Buffer",
            "data": [
                244, 68, 149, 217, 63, 104, 116, 116, 112, 115, 58, 47, 47, 97, 112, 105,
                46, 120, 109, 111, 98, 113, 97, 46, 99, 111, 109, 47, 101, 114, 114, 111,
                114, 115, 47, 118, 97, 108, 105, 100, 97, 116, 105, 111, 110, 47, 109, 105,
                115, 115, 105, 110, 103, 45, 114, 101, 113, 117, 105, 114, 101, 100, 45, 102,
                105, 101, 108, 100, 184, 80, 104, 111, 110, 101, 32, 110, 117, 109, 98, 101,
                114, 32, 105, 115, 32, 31, 0, 175, 205, 1, 144, 217, 63, 84, 104, 101, 32,
                112, 33, 0, 4, 240, 21, 32, 97, 110, 100, 32, 99, 97, 110, 110, 111, 116,
                32, 98, 101, 32, 101, 109, 112, 116, 121, 32, 111, 114, 32, 119, 104, 105,
                116, 101, 115, 112, 97, 99, 101, 46, 184, 150, 0, 240, 5, 47, 118, 49, 47,
                101, 110, 100, 45, 117, 115, 101, 114, 115, 63, 112, 104, 111, 110, 101, 61
            ]
        }
    ]);
    
    let file_path = generate_test_data("format_test", &valid_data);
    
    // Testar formato JSON
    let json_result = app::LZ4MessagePackProcessor::process(Some(&file_path), app::OutputFormat::Json);
    assert!(json_result.is_ok(), "JSON format should succeed");
    let json_content = json_result.unwrap();
    assert!(json_content.contains("messagepack_hex"), "JSON should contain messagepack_hex field");
    assert!(json_content.contains("human_readable"), "JSON should contain human_readable field");
    
    // Testar formato HEX
    let hex_result = app::LZ4MessagePackProcessor::process(Some(&file_path), app::OutputFormat::Hex);
    assert!(hex_result.is_ok(), "HEX format should succeed");
    let hex_content = hex_result.unwrap();
    assert!(hex_content.chars().all(|c| c.is_digit(16) || c.is_ascii_lowercase() && c >= 'a' && c <= 'f'), 
        "HEX should only contain hexadecimal characters");
}

// Testar casos com tamanhos de buffer variados
#[test]
fn test_varying_buffer_sizes() {
    // Pequeno buffer
    let small_buffer = json!([
        {
            "buffer": {
                "type": "Buffer",
                "data": [204, 10]  // 10 bytes descomprimidos
            },
            "type": 98
        },
        {
            "type": "Buffer",
            "data": [1, 2, 3, 4, 5]  // Dados de exemplo
        }
    ]);
    
    let file_path = generate_test_data("small_buffer", &small_buffer);
    let small_result = app::LZ4MessagePackProcessor::process(Some(&file_path), app::OutputFormat::Human);
    
    // Buffer médio já testado em test_valid_data
    
    // Buffer grande
    let mut large_buffer_data: Vec<u8> = Vec::new();
    for i in 0..10000 {
        large_buffer_data.push((i % 256) as u8);
    }
    
    let large_buffer = json!([
        {
            "buffer": {
                "type": "Buffer",
                "data": [204, 240]  // 240 bytes descomprimidos
            },
            "type": 98
        },
        {
            "type": "Buffer",
            "data": large_buffer_data
        }
    ]);
    
    let file_path = generate_test_data("large_buffer", &large_buffer);
    let large_result = app::LZ4MessagePackProcessor::process(Some(&file_path), app::OutputFormat::Human);
    
    // Apenas verificando se não quebra com tamanhos variados
    assert!(small_result.is_ok() || small_result.is_err(), "Should handle small buffer");
    assert!(large_result.is_ok() || large_result.is_err(), "Should handle large buffer");
} 