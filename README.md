# LZ4 MessagePack

A Rust library for compressing JSON data using LZ4 and MessagePack serialization.

## Features

- JSON to MessagePack conversion
- LZ4 compression
- FFI interface for Flutter integration
- Support for complex data structures
- Efficient binary serialization

## Usage

### Rust

```rust
use lz4_messagepack::process_lz4_messagepack;

fn main() {
    let json = r#"
        {
            "name": "John Doe",
            "age": 30,
            "address": {
                "street": "123 Main St",
                "city": "Anytown"
            }
        }
    "#;
    
    let result = process_lz4_messagepack(json);
    println!("{}", result);
}
```

### Flutter

```dart
import 'dart:ffi';
import 'package:ffi/ffi.dart';

// Load the library
final DynamicLibrary nativeLib = Platform.isAndroid
    ? DynamicLibrary.open('liblz4_messagepack.so')
    : DynamicLibrary.process();

// Define native functions
typedef ProcessLz4MessagepackNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef FreeStringNative = Void Function(Pointer<Utf8>);

final processLz4Messagepack = nativeLib
    .lookupFunction<ProcessLz4MessagepackNative, ProcessLz4MessagepackNative>(
        'process_lz4_messagepack');
final freeString = nativeLib
    .lookupFunction<FreeStringNative, FreeStringNative>('free_string');

// Function to process JSON
String processLz4Messagepack(String inputJson) {
  final inputPtr = inputJson.toNativeUtf8();
  final resultPtr = processLz4Messagepack(inputPtr);
  final result = resultPtr.toDartString();
  
  // Free memory
  freeString(resultPtr);
  free(inputPtr);
  
  return result;
}
```

## Building

### Rust Library

```bash
cargo build --release
```

### Flutter Integration

1. Copy the compiled library to your Flutter project:
   - Android: `android/app/src/main/jniLibs/<arch>/liblz4_messagepack.so`
   - iOS: Add to Xcode project

2. Add dependencies to `pubspec.yaml`:
```yaml
dependencies:
  ffi: ^2.0.1
```

## License

This project is licensed under the MIT License - see the LICENSE file for details. 