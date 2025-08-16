# Use Cases Guide

This guide demonstrates real-world scenarios where j2s can streamline your development workflow.

## Table of Contents

1. [API Development](#api-development)
2. [Configuration Management](#configuration-management)
3. [Data Migration](#data-migration)
4. [Testing and Mocking](#testing-and-mocking)
5. [Documentation Generation](#documentation-generation)
6. [Cross-Language Development](#cross-language-development)
7. [Database Schema Design](#database-schema-design)
8. [Microservices Communication](#microservices-communication)

## API Development

### REST API Response Types

**Scenario**: You're building a REST API and need consistent types across frontend and backend.

**Workflow**:
1. Design your API response format in JSON
2. Generate types for both frontend (TypeScript) and backend (Go/Rust)
3. Maintain consistency across your stack

**Example**:
```json
// api_user_response.json
{
  "success": true,
  "data": {
    "user": {
      "id": 12345,
      "username": "johndoe",
      "email": "john@example.com",
      "profile": {
        "display_name": "John Doe",
        "avatar_url": "https://example.com/avatar.jpg",
        "bio": "Software developer"
      },
      "permissions": ["read", "write", "admin"],
      "last_login": "2024-01-15T10:30:00Z"
    },
    "metadata": {
      "request_id": "req_123456",
      "timestamp": "2024-01-15T10:30:00Z"
    }
  },
  "errors": null
}
```

**Generate types**:
```bash
# Backend (Go)
j2s api_user_response.json --format go --struct-name UserResponse --output backend/types/user.go

# Frontend (TypeScript)
j2s api_user_response.json --format typescript --struct-name UserResponse --output frontend/types/user.ts
```

**Usage in Go backend**:
```go
func GetUser(w http.ResponseWriter, r *http.Request) {
    user := fetchUserFromDB(userID)
    
    response := UserResponse{
        Success: true,
        Data: UserResponseData{
            User: user,
            Metadata: Metadata{
                RequestID: generateRequestID(),
                Timestamp: time.Now().Format(time.RFC3339),
            },
        },
        Errors: nil,
    }
    
    json.NewEncoder(w).Encode(response)
}
```

**Usage in TypeScript frontend**:
```typescript
async function fetchUser(id: number): Promise<UserResponse> {
    const response = await fetch(`/api/users/${id}`);
    const data: UserResponse = await response.json();
    
    if (data.success) {
        return data;
    } else {
        throw new Error('Failed to fetch user');
    }
}
```

### GraphQL Schema Generation

**Scenario**: Convert REST API responses to GraphQL schema types.

```bash
# Generate TypeScript types for GraphQL resolvers
j2s user_query_response.json --format typescript --struct-name UserQueryResponse --output graphql/types/user.ts
```

## Configuration Management

### Application Configuration

**Scenario**: Manage complex application configurations with type safety.

**Example Configuration** (`app_config.json`):
```json
{
  "app": {
    "name": "MyApp",
    "version": "1.2.3",
    "environment": "production"
  },
  "server": {
    "host": "0.0.0.0",
    "port": 8080,
    "ssl": {
      "enabled": true,
      "cert_path": "/etc/ssl/cert.pem",
      "key_path": "/etc/ssl/key.pem"
    }
  },
  "database": {
    "host": "localhost",
    "port": 5432,
    "name": "myapp_prod",
    "ssl_mode": "require",
    "pool": {
      "min_connections": 5,
      "max_connections": 20,
      "timeout_seconds": 30
    }
  },
  "features": {
    "authentication": true,
    "rate_limiting": true,
    "caching": false
  },
  "external_services": {
    "redis": {
      "url": "redis://localhost:6379",
      "timeout_ms": 5000
    },
    "email": {
      "provider": "sendgrid",
      "api_key": "${EMAIL_API_KEY}",
      "from_address": "noreply@myapp.com"
    }
  }
}
```

**Generate configuration types**:
```bash
# Go configuration
j2s app_config.json --format go --struct-name AppConfig --output config/types.go

# Rust configuration
j2s app_config.json --format rust --struct-name AppConfig --output src/config.rs
```

**Usage in Go**:
```go
package config

import (
    "encoding/json"
    "os"
)

func LoadConfig(path string) (*AppConfig, error) {
    file, err := os.Open(path)
    if err != nil {
        return nil, err
    }
    defer file.Close()
    
    var config AppConfig
    decoder := json.NewDecoder(file)
    if err := decoder.Decode(&config); err != nil {
        return nil, err
    }
    
    // Validate configuration
    if config.Server.Port <= 0 {
        return nil, errors.New("invalid server port")
    }
    
    return &config, nil
}
```

### Environment-Specific Configs

**Scenario**: Manage different configurations for dev, staging, and production.

```bash
# Generate types for all environments
j2s config/development.json --format go --struct-name Config --output config/dev.go
j2s config/staging.json --format go --struct-name Config --output config/staging.go
j2s config/production.json --format go --struct-name Config --output config/prod.go
```

## Data Migration

### Database Schema Migration

**Scenario**: Migrate data between different database schemas or systems.

**Old Schema** (`old_user_schema.json`):
```json
{
  "id": 123,
  "name": "John Doe",
  "email": "john@example.com",
  "created": "2024-01-15"
}
```

**New Schema** (`new_user_schema.json`):
```json
{
  "user_id": 123,
  "first_name": "John",
  "last_name": "Doe",
  "email_address": "john@example.com",
  "profile": {
    "display_name": "John Doe",
    "created_at": "2024-01-15T00:00:00Z",
    "updated_at": "2024-01-15T00:00:00Z"
  }
}
```

**Generate migration types**:
```bash
j2s old_user_schema.json --format go --struct-name OldUser --output migration/old_user.go
j2s new_user_schema.json --format go --struct-name NewUser --output migration/new_user.go
```

**Migration logic**:
```go
func MigrateUser(old OldUser) NewUser {
    names := strings.Split(old.Name, " ")
    firstName := names[0]
    lastName := ""
    if len(names) > 1 {
        lastName = strings.Join(names[1:], " ")
    }
    
    return NewUser{
        UserID:    old.ID,
        FirstName: firstName,
        LastName:  lastName,
        EmailAddress: old.Email,
        Profile: Profile{
            DisplayName: old.Name,
            CreatedAt:   old.Created + "T00:00:00Z",
            UpdatedAt:   time.Now().Format(time.RFC3339),
        },
    }
}
```

### Data Format Conversion

**Scenario**: Convert data between different formats (CSV to JSON, XML to JSON, etc.).

```bash
# Convert CSV export to typed structures
csv2json users.csv | j2s --format rust --struct-name User --output models/user.rs
```

## Testing and Mocking

### Test Data Generation

**Scenario**: Generate consistent test data structures across your test suite.

**Test Data Schema** (`test_user.json`):
```json
{
  "id": 1,
  "username": "testuser",
  "email": "test@example.com",
  "profile": {
    "first_name": "Test",
    "last_name": "User"
  },
  "permissions": ["read"],
  "created_at": "2024-01-15T10:30:00Z"
}
```

**Generate test types**:
```bash
j2s test_user.json --format go --struct-name TestUser --output tests/fixtures/user.go
j2s test_user.json --format typescript --struct-name TestUser --output tests/fixtures/user.ts
```

**Usage in tests**:
```go
func TestUserCreation(t *testing.T) {
    testUser := TestUser{
        ID:       1,
        Username: "testuser",
        Email:    "test@example.com",
        Profile: TestProfile{
            FirstName: "Test",
            LastName:  "User",
        },
        Permissions: []string{"read"},
        CreatedAt:   "2024-01-15T10:30:00Z",
    }
    
    // Use testUser in your tests...
}
```

### Mock API Responses

**Scenario**: Create mock API responses for frontend development.

```bash
# Generate mock response types
j2s mock_api_response.json --format typescript --struct-name MockResponse --output mocks/api.ts
```

```typescript
// Mock service
export class MockUserService {
    async getUser(id: number): Promise<MockResponse> {
        return {
            success: true,
            data: {
                user: {
                    id: id,
                    username: `user${id}`,
                    email: `user${id}@example.com`,
                    // ... other mock data
                }
            },
            errors: null
        };
    }
}
```

## Documentation Generation

### API Documentation

**Scenario**: Generate API documentation from response schemas.

```bash
# Generate schema for documentation tools
j2s api_responses/*.json --format schema --output docs/schemas/
```

**Integration with OpenAPI**:
```yaml
# openapi.yml
components:
  schemas:
    UserResponse:
      $ref: './schemas/user_response.schema.json'
```

### Code Examples

**Scenario**: Generate code examples for documentation.

```bash
# Generate examples for different languages
j2s example_request.json --format go --struct-name ExampleRequest --output docs/examples/go.go
j2s example_request.json --format python --struct-name ExampleRequest --output docs/examples/python.py
```

## Cross-Language Development

### Polyglot Microservices

**Scenario**: Multiple services in different languages need to share data structures.

**Shared Schema** (`shared_event.json`):
```json
{
  "event_id": "evt_123456",
  "event_type": "user.created",
  "timestamp": "2024-01-15T10:30:00Z",
  "payload": {
    "user_id": 12345,
    "username": "johndoe",
    "email": "john@example.com"
  },
  "metadata": {
    "source": "user-service",
    "version": "1.0"
  }
}
```

**Generate for each service**:
```bash
# User service (Go)
j2s shared_event.json --format go --struct-name Event --output user-service/types/event.go

# Notification service (Python)
j2s shared_event.json --format python --struct-name Event --output notification-service/models/event.py

# Analytics service (Rust)
j2s shared_event.json --format rust --struct-name Event --output analytics-service/src/event.rs

# Frontend (TypeScript)
j2s shared_event.json --format typescript --struct-name Event --output frontend/types/event.ts
```

### Message Queue Integration

**Scenario**: Ensure consistent message formats across producers and consumers.

```bash
# Generate message types for different services
j2s message_schemas/order_created.json --format go --struct-name OrderCreatedEvent --output order-service/events/
j2s message_schemas/order_created.json --format python --struct-name OrderCreatedEvent --output inventory-service/events/
```

## Database Schema Design

### ORM Model Generation

**Scenario**: Generate ORM models from JSON data samples.

**Sample Data** (`product_sample.json`):
```json
{
  "id": 1,
  "name": "Laptop",
  "price": 999.99,
  "category_id": 5,
  "specifications": {
    "cpu": "Intel i7",
    "ram": "16GB",
    "storage": "512GB SSD"
  },
  "tags": ["electronics", "computers"],
  "created_at": "2024-01-15T10:30:00Z"
}
```

**Generate models**:
```bash
# Django models (Python)
j2s product_sample.json --format python --struct-name Product --output models/product.py

# GORM models (Go)
j2s product_sample.json --format go --struct-name Product --output models/product.go
```

**Post-process for ORM**:
```python
# Add Django ORM decorators
from django.db import models

class Product(models.Model):
    name = models.CharField(max_length=255)
    price = models.DecimalField(max_digits=10, decimal_places=2)
    category = models.ForeignKey(Category, on_delete=models.CASCADE)
    specifications = models.JSONField()
    created_at = models.DateTimeField(auto_now_add=True)
    
    class Meta:
        db_table = 'products'
```

### Schema Validation

**Scenario**: Create validation schemas for incoming data.

```bash
# Generate JSON Schema for validation
j2s user_input.json --format schema --output validation/user_schema.json
```

## Microservices Communication

### Service Contracts

**Scenario**: Define contracts between microservices.

**Service Contract** (`user_service_contract.json`):
```json
{
  "service": "user-service",
  "version": "1.0",
  "endpoints": {
    "get_user": {
      "request": {
        "user_id": 12345
      },
      "response": {
        "user": {
          "id": 12345,
          "username": "johndoe",
          "email": "john@example.com"
        }
      }
    }
  }
}
```

**Generate client types**:
```bash
# Generate client types for consuming services
j2s user_service_contract.json --format typescript --struct-name UserServiceContract --output clients/user-service.ts
```

### Event Sourcing

**Scenario**: Define event schemas for event sourcing systems.

```bash
# Generate event types
j2s events/user_events.json --format rust --struct-name UserEvent --output src/events/user.rs
j2s events/order_events.json --format rust --struct-name OrderEvent --output src/events/order.rs
```

## Integration Examples

### CI/CD Pipeline

```yaml
# .github/workflows/generate-types.yml
name: Generate Types
on:
  push:
    paths: ['schemas/**/*.json']

jobs:
  generate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
      - name: Build j2s
        run: cargo build --release
      - name: Generate types
        run: |
          for schema in schemas/*.json; do
            base=$(basename "$schema" .json)
            ./target/release/j2s "$schema" --format go --struct-name "${base^}" --output "generated/go/${base}.go"
            ./target/release/j2s "$schema" --format typescript --struct-name "${base^}" --output "generated/ts/${base}.ts"
          done
      - name: Commit generated code
        run: |
          git add generated/
          git commit -m "Update generated types" || exit 0
          git push
```

### Docker Integration

```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM alpine:latest
RUN apk add --no-cache ca-certificates
COPY --from=builder /app/target/release/j2s /usr/local/bin/
WORKDIR /workspace
ENTRYPOINT ["j2s"]
```

```bash
# Use in CI/CD
docker run --rm -v $(pwd):/workspace j2s-image schemas/user.json --format go --output models/user.go
```

These use cases demonstrate how j2s can be integrated into various development workflows to improve type safety, reduce manual work, and maintain consistency across different parts of your system.