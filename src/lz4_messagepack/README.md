# LZ4 MessagePack Processor

Utilitário em Rust para processar dados MessagePack comprimidos com LZ4BlockArray (tipo 98), como os usados pelo MessagePack para C#.

## Funcionalidades

- Desserialização de JSON contendo dados MessagePack com LZ4BlockArray
- Tentativa de descompressão usando diferentes estratégias
- Conversão para formato JSON legível
- Suporte a vários formatos de saída (JSON, hex, binário, legível)

## Estrutura

O código foi organizado seguindo princípios de Clean Code:

- `OutputFormat`: Enum para os formatos de saída disponíveis
- `MessagePackExt`: Estrutura que representa um bloco de extensão MessagePack
- `LZ4MessagePackProcessor`: Estrutura principal com funcionalidades para processamento

## Uso

```bash
# Compilar o projeto
cargo build --release

# Mostrar ajuda
cargo run --release -- --help

# Usar dados de teste padrão (saída em JSON)
cargo run --release

# Processar um arquivo JSON e exibir em formato legível
cargo run --release -- input.json human

# Processar dados padrão em formato legível
cargo run --release -- human

# Processar JSON da entrada padrão
cat input.json | cargo run --release -- - [formato]

# Gerar saída binária para um arquivo
cargo run --release -- input.json binary > output.msgpack
```

## Formatos de Saída

- `json` (padrão): Gera um objeto JSON com a representação hex e metadados
- `hex`: Exibe apenas a string hexadecimal
- `binary`: Gera dados binários (útil para redirecionamento)
- `human`: Tenta descomprimir e exibir o conteúdo em formato legível

## Formato de Entrada

O JSON de entrada deve conter um array com dois elementos:

1. Um objeto com:
   - `type`: O tipo de extensão MessagePack (98 para LZ4BlockArray)
   - `buffer`: Um objeto com um array `data` contendo os bytes do cabeçalho

2. Um objeto com:
   - `type`: "Buffer"
   - `data`: Um array de bytes representando os dados comprimidos

## Exemplo de Entrada

```json
[
    {
        "buffer": {
            "type": "Buffer",
            "data": [204, 184]
        },
        "type": 98
    },
    {
        "type": "Buffer",
        "data": [244, 68, 149, ...mais bytes...]
    }
]
```

## Exemplo de Saída (formato human)

```json
{
  "detail": "The phone number is required and cannot be empty or whitespace.",
  "instance": "/api/v1/end-users?phone=",
  "status": 400,
  "title": "Phone number is required",
  "type": "https://api.xmobqa.com/errors/validation/missing-required-field",
  "_raw": "Array([String(Utf8String { s: Ok(\"https://api.xmobqa.com/errors/validation/missing-required-field\") }), String(Utf8String { s: Ok(\"Phone number is required\") }), Integer(PosInt(400)), String(Utf8String { s: Ok(\"The phone number is required and cannot be empty or whitespace.\") }), String(Utf8String { s: Ok(\"/api/v1/end-users?phone=\") })])"
}
```

## Notas sobre o Formato LZ4BlockArray

O formato LZ4BlockArray (tipo 98) do MessagePack C# é um formato especial integrado ao pipeline de serialização MessagePack, não apenas dados comprimidos com LZ4. Esta ferramenta tenta descomprimir e interpretar os dados, mas pode não ter sucesso em todos os casos devido às particularidades da implementação C#.