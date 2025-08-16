# Quick Start Guide

Get up and running with j2s in 5 minutes.

## Installation

```bash
git clone https://github.com/echozyr2001/j2s
cd j2s
cargo build --release
```

## Basic Usage

### 1. Create a JSON file

```bash
cat > example.json << 'EOF'
{
  "name": "John Doe",
  "age": 30,
  "email": "john@example.com",
  "active": true,
  "tags": ["developer", "rust"]
}
EOF
```

### 2. Generate code

```bash
# Go structs
./target/release/j2s example.json --format go --struct-name Person --output person.go

# Rust structs  
./target/release/j2s example.json --format rust --struct-name Person --output person.rs

# TypeScript interfaces
./target/release/j2s example.json --format typescript --struct-name Person --output person.ts

# Python dataclasses
./target/release/j2s example.json --format python --struct-name Person --output person.py

# JSON Schema (default)
./target/release/j2s example.json --output person.schema.json
```

### 3. Use the generated code

**Go**:
```go
package main

import (
    "encoding/json"
    "fmt"
)

func main() {
    jsonData := `{"name":"John","age":30,"email":"john@example.com","active":true,"tags":["developer","rust"]}`
    
    var person Person
    json.Unmarshal([]byte(jsonData), &person)
    
    fmt.Printf("Name: %s, Age: %d\n", person.Name, person.Age)
}
```

**Rust**:
```rust
use serde_json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let json_data = r#"{"name":"John","age":30,"email":"john@example.com","active":true,"tags":["developer","rust"]}"#;
    
    let person: Person = serde_json::from_str(json_data)?;
    
    println!("Name: {}, Age: {}", person.name, person.age);
    Ok(())
}
```

**TypeScript**:
```typescript
import { Person } from './person';

const jsonData = '{"name":"John","age":30,"email":"john@example.com","active":true,"tags":["developer","rust"]}';
const person: Person = JSON.parse(jsonData) as Person;

console.log(`Name: ${person.name}, Age: ${person.age}`);
```

**Python**:
```python
import json
from person import Person

json_data = '{"name":"John","age":30,"email":"john@example.com","active":true,"tags":["developer","rust"]}'
data = json.loads(json_data)

person = Person(**data)
print(f"Name: {person.name}, Age: {person.age}")
```

## Command Options

| Option | Short | Description | Example |
|--------|-------|-------------|---------|
| `--format` | `-f` | Target language | `--format go` |
| `--struct-name` | `-s` | Struct/class name | `--struct-name User` |
| `--output` | `-o` | Output file | `--output models.go` |
| `--help` | `-h` | Show help | `--help` |

## Supported Formats

- `go` - Go structs with JSON tags
- `rust` - Rust structs with serde
- `typescript` / `ts` - TypeScript interfaces  
- `python` / `py` - Python dataclasses
- `schema` - JSON Schema (default)

## Next Steps

- Read the [Tutorial](TUTORIAL.md) for detailed examples and real-world usage
- Check [Best Practices](BEST_PRACTICES.md) for optimization tips
- Explore [Use Cases](USE_CASES.md) for integration scenarios
- See [Troubleshooting](TROUBLESHOOTING.md) if you encounter issues

## Common Use Cases

### API Response Types
```bash
curl https://api.example.com/users | j2s --format typescript --struct-name ApiResponse
```

### Configuration Files
```bash
j2s config.json --format go --struct-name Config --output config.go
```

### Database Schemas
```bash
j2s user_schema.json --format rust --struct-name User --output models/user.rs
```

That's it! You're ready to generate type-safe code from JSON data.