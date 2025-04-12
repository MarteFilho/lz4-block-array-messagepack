use serde_json::{json, Value as JsonValue};
use std::fs::{self, File};
use std::io::{self, Write, Read};
use std::path::{Path, PathBuf};
use rmpv::{Value, Integer, Utf8String};
use rmpv::encode::write_value;
use lz4::block::compress;

fn main() -> io::Result<()> {
    // Criar diretório de testes
    let test_dir = Path::new("tests/generated");
    if !test_dir.exists() {
        fs::create_dir_all(test_dir)?;
    }
    
    // Gerar diferentes casos de teste
    generate_empty_data(test_dir)?;
    generate_simple_data(test_dir)?;
    generate_complex_data(test_dir)?;
    generate_string_data(test_dir)?;
    generate_numeric_data(test_dir)?;
    generate_mixed_data(test_dir)?;
    generate_large_data(test_dir)?;
    
    println!("Dados de teste gerados com sucesso em: {}", test_dir.display());
    Ok(())
}

// Função para comprimir dados com LZ4 e gerar o JSON correspondente
fn create_lz4_json_test(name: &str, data: &[u8], directory: &Path) -> io::Result<()> {
    // Comprimir os dados com LZ4
    let compressed_data = compress(data, None, false).unwrap_or_default();
    
    // Calcular o tamanho descomprimido (para o header)
    let uncompressed_size = data.len() as u8;
    
    // Criar o JSON com a estrutura LZ4BlockArray
    let json_data = json!([
        {
            "buffer": {
                "type": "Buffer",
                "data": [204, uncompressed_size]
            },
            "type": 98
        },
        {
            "type": "Buffer",
            "data": compressed_data
        }
    ]);
    
    // Escrever para o arquivo
    let file_path = directory.join(format!("{}.json", name));
    let json_string = serde_json::to_string_pretty(&json_data)?;
    fs::write(&file_path, json_string)?;
    
    // Escrever os dados originais em um arquivo separado para referência
    let raw_path = directory.join(format!("{}.raw", name));
    fs::write(&raw_path, data)?;
    
    println!("Gerado teste '{}' com {} bytes (comprimido: {} bytes)",
            name, data.len(), compressed_data.len());
    
    Ok(())
}

// Gerar teste com dados vazios
fn generate_empty_data(directory: &Path) -> io::Result<()> {
    // Array vazio
    let empty_array = b"[]";
    create_lz4_json_test("empty_array", empty_array, directory)?;
    
    // Objeto vazio
    let empty_object = b"{}";
    create_lz4_json_test("empty_object", empty_object, directory)?;
    
    Ok(())
}

// Gerar teste com dados simples
fn generate_simple_data(directory: &Path) -> io::Result<()> {
    // String simples
    let simple_string = b"Hello, world!";
    create_lz4_json_test("simple_string", simple_string, directory)?;
    
    // Array simples
    let mut buffer = Vec::new();
    let array = Value::Array(vec![
        Value::Integer(Integer::from(1)),
        Value::Integer(Integer::from(2)),
        Value::Integer(Integer::from(3))
    ]);
    write_value(&mut buffer, &array).unwrap();
    create_lz4_json_test("simple_array", &buffer, directory)?;
    
    Ok(())
}

// Gerar teste com dados complexos
fn generate_complex_data(directory: &Path) -> io::Result<()> {
    // Objeto ValidationProblemDetails
    let mut buffer = Vec::new();
    let complex_obj = Value::Array(vec![
        Value::String(Utf8String::from("https://api.example.com/errors/validation/field-required")),
        Value::String(Utf8String::from("Field is required")),
        Value::Integer(Integer::from(400)),
        Value::String(Utf8String::from("The field is required and cannot be empty")),
        Value::String(Utf8String::from("/api/v1/resources?id=123"))
    ]);
    write_value(&mut buffer, &complex_obj).unwrap();
    create_lz4_json_test("validation_error", &buffer, directory)?;
    
    Ok(())
}

// Gerar teste com strings grandes
fn generate_string_data(directory: &Path) -> io::Result<()> {
    // String grande
    let mut long_string = String::from("This is a very long string that should be compressed effectively with LZ4. ");
    for _ in 0..10 {
        long_string = long_string.repeat(2);
    }
    
    create_lz4_json_test("long_string", long_string.as_bytes(), directory)?;
    
    // String com caracteres especiais
    let special_chars = "Caracteres especiais: áéíóú àèìòù âêîôû ãõ ç ñ € ¥ £ © ® ™ ☺ ♥ ♦ ♣ ♠ ⚡ ☀ ☁ ☂ ☃";
    create_lz4_json_test("special_chars", special_chars.as_bytes(), directory)?;
    
    Ok(())
}

// Gerar teste com números
fn generate_numeric_data(directory: &Path) -> io::Result<()> {
    // Array de números
    let mut buffer = Vec::new();
    let mut numbers = Vec::new();
    for i in 0..100 {
        numbers.push(Value::Integer(Integer::from(i)));
    }
    let array = Value::Array(numbers);
    write_value(&mut buffer, &array).unwrap();
    create_lz4_json_test("numeric_array", &buffer, directory)?;
    
    Ok(())
}

// Gerar teste com dados mistos
fn generate_mixed_data(directory: &Path) -> io::Result<()> {
    // Objetos aninhados
    let mut buffer: Vec<u8> = Vec::new();
    let mut map = serde_json::Map::new();
    map.insert("name".to_string(), json!("Product"));
    map.insert("price".to_string(), json!(29.99));
    map.insert("in_stock".to_string(), json!(true));
    map.insert("tags".to_string(), json!(["electronics", "gadget", "sale"]));
    map.insert("details".to_string(), json!({
        "manufacturer": "Example Corp",
        "model": "X123",
        "dimensions": {
            "width": 10,
            "height": 5,
            "depth": 2
        }
    }));
    
    let json_str = serde_json::to_string(&JsonValue::Object(map))?;
    create_lz4_json_test("product_data", json_str.as_bytes(), directory)?;
    
    Ok(())
}

// Gerar teste com dados grandes
fn generate_large_data(directory: &Path) -> io::Result<()> {
    // Gerar um array grande com dados repetitivos (altamente compressível)
    let mut buffer = Vec::new();
    let mut large_array = Vec::new();
    for i in 0..1000 {
        let item = i % 10;
        large_array.push(Value::Integer(Integer::from(item)));
    }
    let array = Value::Array(large_array);
    write_value(&mut buffer, &array).unwrap();
    create_lz4_json_test("large_array", &buffer, directory)?;
    
    // Gerar dados binários grandes (menos compressíveis)
    let mut binary_data = Vec::new();
    for i in 0..10000 {
        binary_data.push((i * 17) as u8);
    }
    create_lz4_json_test("binary_data", &binary_data, directory)?;
    
    Ok(())
} 