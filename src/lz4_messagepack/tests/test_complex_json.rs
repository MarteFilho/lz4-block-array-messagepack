use std::fs::File;
use std::io::Read;
use serde_json::{json, Value};
use lz4::block::compress;
use serde::{Serialize, Deserialize};

/// Função auxiliar para compactação e criação de arquivo de teste
fn create_test_file(name: &str, data: &serde_json::Value) -> String {
    // Converter para string JSON
    let json_str = serde_json::to_string_pretty(data).expect("Falha ao serializar JSON");
    
    // Salvar o JSON original
    let original_file = format!("test_{}_original.json", name);
    std::fs::write(&original_file, &json_str).expect("Falha ao escrever arquivo JSON");
    
    // Converter para MessagePack
    let msgpack_data = serde_json::to_vec(data).expect("Falha ao serializar para bytes");
    
    // Comprimir com LZ4
    let compressed_data = compress(&msgpack_data, None, false).expect("Falha ao comprimir dados");
    
    // Criar estrutura de wrapper 
    let wrapper = json!([
        {
            "type": 98,
            "buffer": {
                "type": "Buffer",
                "data": compressed_data.iter().map(|&b| b as u64).collect::<Vec<_>>()
            }
        },
        {
            "type": "Buffer",
            "data": msgpack_data.iter().map(|&b| b as u64).collect::<Vec<_>>()
        }
    ]);
    
    // Salvar o arquivo de teste
    let file_path = format!("test_{}.json", name);
    std::fs::write(&file_path, serde_json::to_string_pretty(&wrapper).unwrap())
        .expect("Falha ao escrever arquivo de teste");
    
    file_path
}

/// Verifica se os arquivos de teste podem ser processados corretamente
fn test_roundtrip<T>(name: &str, data: &T) 
    where T: Serialize + for<'de> Deserialize<'de> + std::fmt::Debug + PartialEq
{
    println!("Testando roundtrip para {}", name);
    
    // Serializar para JSON
    let json_value = serde_json::to_value(data).expect("Falha ao converter para JSON");
    
    // Criar arquivo de teste
    let file_path = create_test_file(name, &json_value);
    
    // Ler o arquivo novamente
    let mut file = File::open(&file_path).expect("Falha ao abrir arquivo");
    let mut content = String::new();
    file.read_to_string(&mut content).expect("Falha ao ler arquivo");
    
    // Processar o arquivo
    let json_data: Value = serde_json::from_str(&content).expect("Falha ao analisar JSON");
    
    // Extrair dados comprimidos
    if let Some(blocks) = json_data.as_array() {
        if blocks.len() >= 2 {
            // Obter dados comprimidos
            let compressed_data: Vec<u8> = blocks[0]["buffer"]["data"].as_array()
                .unwrap()
                .iter()
                .filter_map(|v| v.as_u64().map(|n| n as u8))
                .collect();
            
            // Obter tamanho original para descompressão
            let msgpack_size = blocks[1]["data"].as_array().unwrap().len();
            
            // Descomprimir
            let decompressed = lz4::block::decompress(&compressed_data, Some(msgpack_size as i32))
                .expect("Falha ao descomprimir dados");
            
            // Desserializar de volta para o objeto
            let deserialized: T = serde_json::from_slice(&decompressed)
                .expect("Falha ao desserializar dados");
            
            // Verificar se os dados são iguais ao original
            assert_eq!(&deserialized, data, "Os dados desserializados devem ser iguais aos originais");
            println!("✅ Teste para {} passou com sucesso!", name);
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct ContactInfo {
    email: String,
    phone: Option<String>,
    addresses: Vec<Address>,
    preferred_contact_method: String,
    emergency_contacts: Vec<EmergencyContact>,
    social_media: std::collections::HashMap<String, String>,
    verified: bool
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Address {
    street: String,
    city: String,
    state: String,
    country: String,
    zip: String,
    coordinates: Option<Coordinates>,
    is_primary: bool
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Coordinates {
    latitude: f64,
    longitude: f64
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct EmergencyContact {
    name: String,
    relationship: String,
    phone: String
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Product {
    id: String,
    name: String,
    description: String,
    price: f64,
    inventory: i32,
    category: Vec<String>,
    ratings: Vec<Rating>,
    tags: Vec<String>,
    attributes: std::collections::HashMap<String, serde_json::Value>,
    discount: Option<Discount>,
    dimensions: Dimensions,
    related_products: Vec<String>,
    images: Vec<Image>,
    created_at: String,
    updated_at: String
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Rating {
    user_id: String,
    score: f32,
    comment: Option<String>,
    date: String
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Discount {
    percentage: f32,
    start_date: String,
    end_date: String,
    code: Option<String>
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Dimensions {
    width: f64,
    height: f64,
    depth: f64,
    weight: f64,
    unit: String
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Image {
    url: String,
    alt_text: String,
    width: i32,
    height: i32,
    is_primary: bool,
    tags: Vec<String>
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct NestedStructure {
    level1: Level1,
    array_of_objects: Vec<SimpleObject>,
    mixed_array: Vec<serde_json::Value>,
    binary_data: Vec<u8>,
    metadata: std::collections::HashMap<String, serde_json::Value>
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Level1 {
    name: String,
    level2: Level2,
    siblings: Vec<Level2>
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Level2 {
    id: i32,
    level3: Level3,
    data: Vec<String>
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Level3 {
    value: String,
    active: bool,
    level4: Option<Level4>
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Level4 {
    key: String,
    value: serde_json::Value
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct SimpleObject {
    id: i32,
    name: String
}

#[test]
fn test_contact_info() {
    let mut social_media = std::collections::HashMap::new();
    social_media.insert("twitter".to_string(), "@usuario_teste".to_string());
    social_media.insert("linkedin".to_string(), "linkedin.com/in/usuario_teste".to_string());
    
    let contact = ContactInfo {
        email: "test@example.com".to_string(),
        phone: Some("+55 11 98765-4321".to_string()),
        addresses: vec![
            Address {
                street: "Rua das Flores, 123".to_string(),
                city: "São Paulo".to_string(),
                state: "SP".to_string(),
                country: "Brasil".to_string(),
                zip: "01234-567".to_string(),
                coordinates: Some(Coordinates {
                    latitude: -23.550520,
                    longitude: -46.633308
                }),
                is_primary: true
            },
            Address {
                street: "Av. Paulista, 1000".to_string(),
                city: "São Paulo".to_string(),
                state: "SP".to_string(),
                country: "Brasil".to_string(),
                zip: "01310-100".to_string(),
                coordinates: None,
                is_primary: false
            }
        ],
        preferred_contact_method: "email".to_string(),
        emergency_contacts: vec![
            EmergencyContact {
                name: "João Silva".to_string(),
                relationship: "Pai".to_string(),
                phone: "+55 11 91234-5678".to_string()
            },
            EmergencyContact {
                name: "Maria Souza".to_string(),
                relationship: "Cônjuge".to_string(),
                phone: "+55 11 99876-5432".to_string()
            }
        ],
        social_media,
        verified: true
    };
    
    test_roundtrip("contact_info", &contact);
}

#[test]
fn test_complex_product() {
    let mut attributes = std::collections::HashMap::new();
    attributes.insert("color".to_string(), json!("blue"));
    attributes.insert("size".to_string(), json!("medium"));
    attributes.insert("materials".to_string(), json!(["cotton", "polyester"]));
    attributes.insert("specs".to_string(), json!({
        "waterproof": true,
        "resistance": "high",
        "certifications": ["ISO9001", "ABNT123"]
    }));
    
    let product = Product {
        id: "PROD-12345".to_string(),
        name: "Produto Super Premium Deluxe".to_string(),
        description: "Este é um produto de altíssima qualidade com muitas funcionalidades".to_string(),
        price: 1299.99,
        inventory: 42,
        category: vec!["Eletrônicos".to_string(), "Gadgets".to_string(), "Premium".to_string()],
        ratings: vec![
            Rating {
                user_id: "USER123".to_string(),
                score: 4.5,
                comment: Some("Ótimo produto, recomendo!".to_string()),
                date: "2023-06-15T14:30:00Z".to_string()
            },
            Rating {
                user_id: "USER456".to_string(),
                score: 5.0,
                comment: Some("O melhor que já comprei!".to_string()),
                date: "2023-07-20T09:15:00Z".to_string()
            },
            Rating {
                user_id: "USER789".to_string(),
                score: 3.0,
                comment: None,
                date: "2023-08-05T18:45:00Z".to_string()
            }
        ],
        tags: vec!["premium".to_string(), "durável".to_string(), "tecnologia".to_string(), "inovador".to_string()],
        attributes,
        discount: Some(Discount {
            percentage: 15.0,
            start_date: "2023-10-01T00:00:00Z".to_string(),
            end_date: "2023-10-31T23:59:59Z".to_string(),
            code: Some("DESCONTO15".to_string())
        }),
        dimensions: Dimensions {
            width: 45.5,
            height: 30.0,
            depth: 12.8,
            weight: 2.35,
            unit: "cm/kg".to_string()
        },
        related_products: vec!["PROD-567".to_string(), "PROD-890".to_string(), "PROD-123".to_string()],
        images: vec![
            Image {
                url: "https://example.com/products/12345-1.jpg".to_string(),
                alt_text: "Vista frontal do produto".to_string(),
                width: 1200,
                height: 800,
                is_primary: true,
                tags: vec!["frontal".to_string(), "detalhe".to_string()]
            },
            Image {
                url: "https://example.com/products/12345-2.jpg".to_string(),
                alt_text: "Vista lateral do produto".to_string(),
                width: 1200,
                height: 800,
                is_primary: false,
                tags: vec!["lateral".to_string()]
            },
            Image {
                url: "https://example.com/products/12345-3.jpg".to_string(),
                alt_text: "Produto em uso".to_string(),
                width: 1200,
                height: 800,
                is_primary: false,
                tags: vec!["uso".to_string(), "contexto".to_string()]
            }
        ],
        created_at: "2023-01-15T10:00:00Z".to_string(),
        updated_at: "2023-09-20T15:30:00Z".to_string()
    };
    
    test_roundtrip("complex_product", &product);
}

#[test]
fn test_deeply_nested_structure() {
    let mut metadata = std::collections::HashMap::new();
    metadata.insert("version".to_string(), json!("1.0.0"));
    metadata.insert("author".to_string(), json!("Test User"));
    metadata.insert("settings".to_string(), json!({
        "theme": "dark",
        "notifications": {
            "email": true,
            "push": false,
            "frequency": "daily"
        },
        "permissions": ["read", "write", "delete"]
    }));
    
    let nested = NestedStructure {
        level1: Level1 {
            name: "Root Object".to_string(),
            level2: Level2 {
                id: 1,
                level3: Level3 {
                    value: "Level 3 Value".to_string(),
                    active: true,
                    level4: Some(Level4 {
                        key: "deepest_key".to_string(),
                        value: json!({
                            "really_deep": {
                                "extremely_deep": {
                                    "impossibly_deep": {
                                        "final_value": 42,
                                        "final_array": [1, 2, 3, 4, 5],
                                        "final_object": {
                                            "a": "A",
                                            "b": "B",
                                            "c": ["C1", "C2", "C3"]
                                        }
                                    }
                                }
                            }
                        })
                    })
                },
                data: vec!["A".to_string(), "B".to_string(), "C".to_string()]
            },
            siblings: vec![
                Level2 {
                    id: 2,
                    level3: Level3 {
                        value: "Sibling 1".to_string(),
                        active: false,
                        level4: None
                    },
                    data: vec!["X".to_string(), "Y".to_string()]
                },
                Level2 {
                    id: 3,
                    level3: Level3 {
                        value: "Sibling 2".to_string(),
                        active: true,
                        level4: Some(Level4 {
                            key: "sibling_key".to_string(),
                            value: json!(["array", "of", "values"])
                        })
                    },
                    data: vec!["1".to_string(), "2".to_string(), "3".to_string()]
                }
            ]
        },
        array_of_objects: vec![
            SimpleObject { id: 1, name: "First".to_string() },
            SimpleObject { id: 2, name: "Second".to_string() },
            SimpleObject { id: 3, name: "Third".to_string() }
        ],
        mixed_array: vec![
            json!(1),
            json!("string"),
            json!(true),
            json!(null),
            json!([1, 2, 3]),
            json!({"a": 1, "b": 2})
        ],
        binary_data: vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
        metadata
    };
    
    test_roundtrip("nested_structure", &nested);
}

#[test]
fn test_array_of_arrays() {
    let arrays = vec![
        vec![1, 2, 3, 4, 5],
        vec![10, 20, 30, 40, 50],
        vec![100, 200, 300, 400, 500],
        vec![1000, 2000, 3000, 4000, 5000]
    ];
    
    test_roundtrip("array_of_arrays", &arrays);
}

#[test]
fn test_complex_maps() {
    let mut outer_map = std::collections::HashMap::new();
    
    // Mapa 1: Strings para Valores
    let mut map1 = std::collections::HashMap::new();
    map1.insert("key1".to_string(), json!(42));
    map1.insert("key2".to_string(), json!("value"));
    map1.insert("key3".to_string(), json!(true));
    
    // Mapa 2: Strings para Arrays
    let mut map2 = std::collections::HashMap::new();
    map2.insert("array1".to_string(), json!([1, 2, 3]));
    map2.insert("array2".to_string(), json!(["a", "b", "c"]));
    map2.insert("array3".to_string(), json!([true, false, true]));
    
    // Mapa 3: Strings para Objetos
    let mut map3 = std::collections::HashMap::new();
    map3.insert("obj1".to_string(), json!({"a": 1, "b": 2}));
    map3.insert("obj2".to_string(), json!({"name": "Test", "active": true}));
    
    // Mapa 4: Strings para Mapas (aninhamento)
    let mut map4 = std::collections::HashMap::new();
    let mut nested_map = std::collections::HashMap::new();
    nested_map.insert("n1".to_string(), json!(10));
    nested_map.insert("n2".to_string(), json!(20));
    map4.insert("nested1".to_string(), json!(nested_map));
    
    // Adicionando todos ao mapa externo
    outer_map.insert("simple_values".to_string(), json!(map1));
    outer_map.insert("arrays".to_string(), json!(map2));
    outer_map.insert("objects".to_string(), json!(map3));
    outer_map.insert("nested_maps".to_string(), json!(map4));
    
    // Adicionando um valor complexo
    outer_map.insert("complex".to_string(), json!({
        "mixed": [1, "string", true, {"key": "value"}],
        "deep": {
            "deeper": {
                "deepest": [
                    {"id": 1, "data": {"x": 1, "y": 2}},
                    {"id": 2, "data": {"x": 3, "y": 4}}
                ]
            }
        }
    }));
    
    test_roundtrip("complex_maps", &outer_map);
} 