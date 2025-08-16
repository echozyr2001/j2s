# Best Practices Guide

This guide provides recommendations for effectively using j2s to generate high-quality code from JSON data.

## JSON Structure Design

### 1. Consistent Naming Conventions

**Good**:
```json
{
  "user_id": 12345,
  "first_name": "John",
  "last_name": "Doe",
  "email_address": "john@example.com",
  "is_active": true,
  "created_at": "2024-01-15T10:30:00Z"
}
```

**Avoid**:
```json
{
  "userId": 12345,
  "FirstName": "John",
  "last-name": "Doe",
  "Email": "john@example.com",
  "active": true,
  "createdAt": "2024-01-15T10:30:00Z"
}
```

**Why**: Consistent naming makes generated code more predictable and easier to work with across different languages.

### 2. Meaningful Field Names

**Good**:
```json
{
  "product_id": "PROD-12345",
  "product_name": "Wireless Headphones",
  "unit_price": 199.99,
  "stock_quantity": 150,
  "is_available": true
}
```

**Avoid**:
```json
{
  "id": "PROD-12345",
  "name": "Wireless Headphones",
  "price": 199.99,
  "qty": 150,
  "avail": true
}
```

**Why**: Descriptive names help j2s generate better comments and make the code self-documenting.

### 3. Consistent Data Types

**Good**:
```json
{
  "users": [
    {
      "id": 1,
      "name": "Alice",
      "age": 30
    },
    {
      "id": 2,
      "name": "Bob",
      "age": 25
    }
  ]
}
```

**Avoid**:
```json
{
  "users": [
    {
      "id": 1,
      "name": "Alice",
      "age": 30
    },
    {
      "id": "2",
      "name": "Bob",
      "age": "25"
    }
  ]
}
```

**Why**: Consistent types prevent union types and make generated code more type-safe.

### 4. Avoid Deep Nesting

**Good** (max 3-4 levels):
```json
{
  "user": {
    "profile": {
      "contact": {
        "email": "user@example.com"
      }
    }
  }
}
```

**Avoid** (too deep):
```json
{
  "data": {
    "response": {
      "payload": {
        "content": {
          "user": {
            "details": {
              "profile": {
                "contact": {
                  "email": "user@example.com"
                }
              }
            }
          }
        }
      }
    }
  }
}
```

**Why**: Deep nesting makes generated code harder to read and can cause performance issues.

## Code Generation Best Practices

### 1. Use Descriptive Struct Names

```bash
# Good
j2s user_data.json --format go --struct-name UserProfile

# Avoid
j2s user_data.json --format go --struct-name Data
```

### 2. Organize Generated Code

```bash
# Create language-specific directories
mkdir -p generated/{go,rust,typescript,python}

# Generate to appropriate directories
j2s data.json --format go --output generated/go/models.go
j2s data.json --format rust --output generated/rust/models.rs
j2s data.json --format typescript --output generated/typescript/models.ts
j2s data.json --format python --output generated/python/models.py
```

### 3. Version Control Strategy

```gitignore
# .gitignore

# Include source JSON files
*.json

# Include generated code (recommended)
generated/

# Or exclude generated code and regenerate in CI
# generated/
```

**Recommendation**: Include generated code in version control to track changes and ensure consistency across environments.

## Language-Specific Best Practices

### Go

#### Project Structure
```
project/
├── models/
│   ├── user.go
│   ├── product.go
│   └── api_response.go
├── main.go
└── go.mod
```

#### Usage Example
```go
package main

import (
    "encoding/json"
    "fmt"
    "log"
    
    "your-project/models"
)

func main() {
    jsonData := `{"user_id": 123, "name": "John"}`
    
    var user models.User
    if err := json.Unmarshal([]byte(jsonData), &user); err != nil {
        log.Fatal(err)
    }
    
    fmt.Printf("User: %+v\n", user)
}
```

#### Dependencies
```bash
# No additional dependencies needed for basic JSON handling
go mod tidy
```

### Rust

#### Project Structure
```
project/
├── src/
│   ├── models/
│   │   ├── mod.rs
│   │   ├── user.rs
│   │   └── product.rs
│   └── main.rs
└── Cargo.toml
```

#### Cargo.toml
```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

#### Usage Example
```rust
use serde_json;
use crate::models::User;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let json_data = r#"{"user_id": 123, "name": "John"}"#;
    
    let user: User = serde_json::from_str(json_data)?;
    println!("User: {:?}", user);
    
    Ok(())
}
```

### TypeScript

#### Project Structure
```
project/
├── src/
│   ├── models/
│   │   ├── user.ts
│   │   ├── product.ts
│   │   └── index.ts
│   └── main.ts
├── package.json
└── tsconfig.json
```

#### Usage Example
```typescript
import { User } from './models/user';

const jsonData = '{"user_id": 123, "name": "John"}';
const user: User = JSON.parse(jsonData) as User;

console.log('User:', user);

// Type-safe access
console.log(`User ID: ${user.user_id}`);
console.log(`Name: ${user.name}`);
```

#### Type Guards (Recommended)
```typescript
function isUser(obj: any): obj is User {
    return obj && 
           typeof obj.user_id === 'number' &&
           typeof obj.name === 'string';
}

const parsed = JSON.parse(jsonData);
if (isUser(parsed)) {
    // Now TypeScript knows it's a User
    console.log(parsed.user_id);
}
```

### Python

#### Project Structure
```
project/
├── models/
│   ├── __init__.py
│   ├── user.py
│   └── product.py
├── main.py
└── requirements.txt
```

#### Usage Example
```python
import json
from models.user import User

json_data = '{"user_id": 123, "name": "John"}'
data = json.loads(json_data)

# Create instance from dict
user = User(**data)
print(f"User: {user}")

# Access fields
print(f"User ID: {user.user_id}")
print(f"Name: {user.name}")
```

#### Validation (Recommended)
```python
from dataclasses import dataclass
from typing import Optional
import json

@dataclass
class User:
    user_id: int
    name: str
    email: Optional[str] = None
    
    def __post_init__(self):
        if self.user_id <= 0:
            raise ValueError("user_id must be positive")
        if not self.name.strip():
            raise ValueError("name cannot be empty")
```

## Testing Generated Code

### 1. Validation Tests

Create tests to ensure generated code works correctly:

```bash
# Go
go test ./models/...

# Rust
cargo test

# TypeScript
npm test

# Python
python -m pytest tests/
```

### 2. Sample Test Cases

#### Go Test Example
```go
func TestUserSerialization(t *testing.T) {
    user := User{
        UserID: 123,
        Name:   "John Doe",
    }
    
    data, err := json.Marshal(user)
    assert.NoError(t, err)
    
    var decoded User
    err = json.Unmarshal(data, &decoded)
    assert.NoError(t, err)
    assert.Equal(t, user, decoded)
}
```

## Performance Optimization

### 1. Large Files

For files >10MB:
```bash
# Monitor progress
j2s large_file.json --format go --struct-name LargeData

# Consider splitting large arrays
jq '.items[0:1000]' large_file.json > sample.json
j2s sample.json --format go
```

### 2. Batch Processing

```bash
#!/bin/bash
# Process multiple files
for file in data/*.json; do
    base=$(basename "$file" .json)
    j2s "$file" --format go --struct-name "${base^}" --output "models/${base}.go"
done
```

### 3. CI/CD Integration

```yaml
# GitHub Actions example
name: Generate Code
on: [push]
jobs:
  generate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Install j2s
        run: cargo install --path .
      - name: Generate code
        run: |
          j2s schemas/user.json --format go --output models/user.go
          j2s schemas/product.json --format rust --output src/models/product.rs
      - name: Commit generated code
        run: |
          git add models/ src/models/
          git commit -m "Update generated code" || exit 0
          git push
```

## Error Handling

### 1. Graceful Degradation

```go
// Go example
func ParseUser(data []byte) (*User, error) {
    var user User
    if err := json.Unmarshal(data, &user); err != nil {
        return nil, fmt.Errorf("failed to parse user: %w", err)
    }
    return &user, nil
}
```

### 2. Validation

```rust
// Rust example
impl User {
    pub fn validate(&self) -> Result<(), String> {
        if self.user_id <= 0 {
            return Err("user_id must be positive".to_string());
        }
        if self.name.is_empty() {
            return Err("name cannot be empty".to_string());
        }
        Ok(())
    }
}
```

## Maintenance

### 1. Regular Updates

- Regenerate code when JSON schemas change
- Update dependencies regularly
- Review generated code for quality
- Run tests after regeneration

### 2. Documentation

- Document your JSON schemas
- Maintain examples for each generated type
- Keep troubleshooting notes
- Document custom modifications

### 3. Monitoring

- Track generated code size
- Monitor compilation times
- Watch for breaking changes
- Validate against real data regularly

## Common Pitfalls to Avoid

1. **Inconsistent field types** - Leads to union types
2. **Reserved keywords as field names** - Causes compilation errors
3. **Overly complex nesting** - Makes code hard to use
4. **Missing null handling** - Runtime errors
5. **Ignoring generated comments** - Loses valuable context
6. **Not testing generated code** - Hidden bugs
7. **Hardcoding struct names** - Reduces reusability
8. **Mixing naming conventions** - Inconsistent APIs

Following these best practices will help you generate high-quality, maintainable code from your JSON data structures.