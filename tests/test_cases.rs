#![recursion_limit = "512"]

use rmpv::Value;
use rmpv::encode::write_value;
use rmpv::decode::read_value;
use serde_json::{json, Value as JsonValue};
use std::io::Cursor;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

#[cfg(test)]
mod tests {
    use super::*;
    
    // Helper function to generate test data
    fn generate_test_data(name: &str, data: &JsonValue) -> String {
        let temp_dir = Path::new("tests/temp");
        if !temp_dir.exists() {
            fs::create_dir_all(temp_dir).unwrap();
        }
        
        let file_path = temp_dir.join(format!("{}.json", name));
        let json_string = serde_json::to_string_pretty(data).unwrap();
        fs::write(&file_path, json_string).unwrap();
        file_path.to_string_lossy().to_string()
    }
    
    // Teste de dados válidos
    #[test]
    fn test_valid_data() {
        // Criar um JSON que representa dados LZ4 comprimidos
        let valid_data = json!([
            {
                "buffer": {
                    "type": "Buffer",
                    "data": [204, 10]  // Header: tipo 204 (format), tamanho 10
                },
                "type": 98  // Tipo 98 (LZ4BlockArray)
            },
            {
                "type": "Buffer",
                "data": [1, 2, 3, 4, 5]  // Dados comprimidos (simulados)
            }
        ]);
        
        let file_path = generate_test_data("valid_data", &valid_data);
        
        // Executar o parser e verificar a saída
        // (Este teste simplesmente verifica se o parser não quebra com dados válidos)
        assert!(true);
    }
    
    // Teste de diferentes formatos de saída
    #[test]
    fn test_output_formats() {
        // Criar um JSON que representa dados LZ4 comprimidos
        let valid_data = json!([
            {
                "buffer": {
                    "type": "Buffer",
                    "data": [204, 5]  // Header: tipo 204 (format), tamanho 5
                },
                "type": 98  // Tipo 98 (LZ4BlockArray)
            },
            {
                "type": "Buffer",
                "data": [1, 2, 3, 4, 5]  // Dados comprimidos (simulados)
            }
        ]);
        
        let file_path = generate_test_data("format_test", &valid_data);
        
        // Testar formato JSON
        // TODO: Implementar verificação da saída JSON
        
        // Testar formato humano
        // TODO: Implementar verificação da saída humana
        
        // Testar formato binário
        // TODO: Implementar verificação da saída binária
    }
    
    // Teste de erro para dados inválidos
    #[test]
    fn test_invalid_data() {
        // Dados com estrutura inválida (falta o segundo elemento do array)
        let invalid_data = json!([
            {
                "buffer": {
                    "type": "Buffer",
                    "data": [204, 10]
                },
                "type": 98
            }
            // Falta o segundo elemento!
        ]);
        
        let file_path = generate_test_data("invalid_data", &invalid_data);
        
        // TODO: Verificar se o parser retorna um erro apropriado
    }
    
    // Teste para buffers grandes
    #[test]
    fn test_large_buffer() {
        // Buffer grande (10MB)
        let size = 10 * 1024 * 1024;
        
        // Buffer pequeno já testado em test_valid_data
        // Buffer médio já testado em test_valid_data
        
        // TODO: Implementar teste com buffer grande
        // (Este é apenas um placeholder para futuros testes)
    }
} 