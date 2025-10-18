# Miru Backend Architecture

**Last Updated**: 2025-01-18  
**Architecture Style**: Domain-Driven Design (DDD) + Clean Architecture + Hexagonal Architecture (Ports & Adapters)

---

## Table of Contents

1. [Overview](#overview)
2. [Architecture Principles](#architecture-principles)
3. [Layer Structure](#layer-structure)
4. [Module Organization](#module-organization)
5. [Design Patterns](#design-patterns)
6. [Data Flow](#data-flow)
7. [Repository Pattern](#repository-pattern)
8. [Domain Events](#domain-events)
9. [Testing Strategy](#testing-strategy)
10. [Development Guidelines](#development-guidelines)

---

## Overview

Miru is a desktop anime management application built with Tauri (Rust backend, TypeScript frontend). The backend follows industry-standard architectural patterns to ensure maintainability, testability, and scalability.

### Key Architectural Decisions

- **DDD (Domain-Driven Design)**: Business logic encapsulated in domain entities and aggregates
- **Clean Architecture**: Strict layer separation with dependency inversion
- **Hexagonal Architecture**: Infrastructure isolated via ports (interfaces)
- **CQRS Pattern**: Commands and queries separated for clarity
- **Event Sourcing**: Domain events track all state changes
- **Repository Pattern**: Data access abstracted behind clean interfaces

---

## Architecture Principles

### 1. **Dependency Rule**

Dependencies point **inward** only. Outer layers depend on inner layers, never the reverse.

```
Presentation → Application → Domain
       ↓             ↓           ↓
Infrastructure ← ← ← ← ← ← ← ← Core
```

### 2. **Separation of Concerns**

Each layer has a single, well-defined responsibility:

- **Domain**: Business logic, entities, value objects, domain events
- **Application**: Use cases, orchestration, DTOs, ports (interfaces)
- **Infrastructure**: Database, external APIs, file system, adapters
- **Presentation**: Tauri commands, API endpoints, DTOs

### 3. **Dependency Inversion**

High-level modules (application) depend on abstractions (ports), not concrete implementations (repositories).

```rust
// Application defines the interface
pub trait AnimeRepository {
    async fn save(&self, aggregate: &AnimeAggregate) -> AppResult<()>;
}

// Infrastructure implements the interface
pub struct AnimeRepositoryImpl { /* ... */ }
impl AnimeRepository for AnimeRepositoryImpl { /* ... */ }
```

### 4. **Screaming Architecture**

Folder structure reveals what the application **does**, not what frameworks it uses.

```
✅ Good: modules/anime/application/use_cases/create_anime/
❌ Bad:  modules/anime/controllers/anime_controller.rs
```

---

## Layer Structure

### **Domain Layer** (`modules/*/domain/`)

**Purpose**: Core business logic, independent of frameworks and infrastructure.

**Contains**:
- **Entities**: Business objects with identity (`AnimeDetailed`, `Genre`, `Studio`)
- **Aggregates**: Consistency boundaries with encapsulation (`AnimeAggregate`)
- **Value Objects**: Immutable objects without identity (`AnimeTitle`, `ProviderMetadata`)
- **Domain Events**: State changes (`AnimeCreatedEvent`, `ScoreUpdatedEvent`)
- **Repository Interfaces**: Contracts for data access (no implementations!)
- **Domain Services**: Cross-entity business logic

**Rules**:
- ✅ No database code (Diesel, SQL)
- ✅ No external API calls (reqwest, HTTP)
- ✅ No framework dependencies (Tauri, Axum)
- ✅ Pure business logic only

**Example**:
```rust
// modules/anime/domain/aggregates/anime_aggregate/anime.rs
pub struct AnimeAggregate {
    entity: AnimeDetailed,
    pending_events: Vec<Box<dyn DomainEvent>>,
}

impl AnimeAggregate {
    pub fn update_score(&mut self, new_score: f32) -> Result<(), String> {
        // Validate business rule
        if new_score < 0.0 || new_score > 10.0 {
            return Err("Score must be between 0-10".to_string());
        }
        
        let old_score = self.entity.score;
        self.entity.score = Some(new_score);
        
        // Publish domain event
        self.pending_events.push(Box::new(
            AnimeScoreUpdatedEvent::new(self.entity.id, old_score, new_score)
        ));
        
        Ok(())
    }
}
```

---

### **Application Layer** (`modules/*/application/`)

**Purpose**: Orchestrate domain objects to implement use cases. Defines interfaces for infrastructure.

**Contains**:
- **Use Cases**: Command handlers (write operations)
- **Queries**: Query handlers (read operations)
- **Ports**: Interfaces for repositories, external services, event publishers
- **DTOs**: Data transfer objects for cross-layer communication
- **Application Services**: Orchestration of multiple use cases

**Rules**:
- ✅ Orchestrates domain objects
- ✅ Defines interfaces (ports) for infrastructure
- ✅ No database implementation details
- ✅ No framework-specific code

**Example**:
```rust
// modules/anime/application/use_cases/update_anime_score/handler.rs
pub struct UpdateAnimeScoreHandler {
    anime_repository: Arc<dyn AnimeRepository>,  // Port (interface)
    event_publisher: Arc<dyn EventPublisher>,    // Port (interface)
}

impl UseCase<UpdateAnimeScoreCommand, UpdateAnimeScoreResult> for UpdateAnimeScoreHandler {
    async fn execute(&self, command: UpdateAnimeScoreCommand) -> AppResult<UpdateAnimeScoreResult> {
        // 1. Load aggregate from repository
        let anime = self.anime_repository.find_by_id(command.anime_id).await?;
        let mut aggregate = AnimeAggregate::from_entity(anime);
        
        // 2. Execute domain logic
        aggregate.update_score(command.new_score)?;
        
        // 3. Persist changes
        self.anime_repository.update(&aggregate).await?;
        
        // 4. Publish domain events
        let events = aggregate.pending_events();
        self.event_publisher.publish_all(events).await?;
        
        Ok(UpdateAnimeScoreResult::new(command.anime_id, command.new_score))
    }
}
```

---

### **Infrastructure Layer** (`modules/*/infrastructure/`)

**Purpose**: Implement interfaces defined by application layer. Handle external concerns.

**Contains**:
- **Repository Implementations**: Database access (Diesel, PostgreSQL)
- **External Service Adapters**: API clients (Jikan, AniList, Kitsu)
- **Persistence Models**: Database-specific structs
- **Mappers**: Convert between domain entities and database models
- **Event Publishers**: Message queues, event stores

**Rules**:
- ✅ Implements ports defined in application layer
- ✅ Contains framework-specific code (Diesel, reqwest)
- ✅ Never accessed directly by domain layer

**Example**:
```rust
// modules/anime/infrastructure/persistence/repositories/anime_repository_impl.rs
pub struct AnimeRepositoryImpl {
    db: Arc<Database>,
}

#[async_trait]
impl AnimeRepository for AnimeRepositoryImpl {  // Implements port
    async fn save(&self, aggregate: &AnimeAggregate) -> AppResult<()> {
        let db = Arc::clone(&self.db);
        let entity = aggregate.entity().clone();
        
        task::spawn_blocking(move || {
            let mut conn = db.get_connection()?;
            conn.transaction(|conn| {
                // Diesel-specific database code
                diesel::insert_into(anime::table)
                    .values(&entity_to_new_model(&entity))
                    .execute(conn)?;
                Ok(())
            })
        }).await?
    }
}
```

---

### **Presentation Layer** (`modules/*/commands/`, `commands/`)

**Purpose**: Expose use cases to external interfaces (Tauri commands, HTTP endpoints).

**Contains**:
- **Tauri Commands**: Desktop app commands
- **Request/Response DTOs**: API-specific data structures
- **Command Registry**: Centralized command registration

**Rules**:
- ✅ Thin layer - delegates to use cases immediately
- ✅ Converts external DTOs to application commands
- ✅ Handles serialization/deserialization

**Example**:
```rust
// modules/anime/commands.rs
#[tauri::command]
#[specta::specta]
pub async fn get_anime_by_id(
    request: GetAnimeByIdRequest,
    anime_service: State<'_, Arc<AnimeService>>,
) -> Result<Option<AnimeDetailed>, String> {
    let anime_id = Uuid::parse_str(&request.id)
        .map_err(|e| format!("Invalid UUID: {}", e))?;
    
    anime_service
        .get_anime_by_id(&anime_id)
        .await
        .map_err(|e| e.to_string())
}
```

---

## Module Organization

Each module follows the same layered structure:

```
modules/anime/
├── domain/
│   ├── aggregates/           # Aggregate roots
│   │   └── anime_aggregate/
│   │       ├── anime.rs      # AnimeAggregate implementation
│   │       ├── relations.rs  # Child entities
│   │       └── mod.rs
│   ├── entities/             # Domain entities
│   │   ├── anime_detailed/
│   │   ├── genre.rs
│   │   └── mod.rs
│   ├── events/               # Domain events
│   │   ├── anime_events.rs
│   │   └── mod.rs
│   ├── value_objects/        # Value objects
│   │   ├── anime_title.rs
│   │   ├── quality_metrics.rs
│   │   └── mod.rs
│   ├── repositories/         # Repository interfaces
│   │   └── anime_repository.rs
│   ├── services/             # Domain services
│   └── traits/
├── application/
│   ├── use_cases/            # Use case handlers (CQRS commands)
│   │   ├── create_anime/
│   │   │   ├── command.rs    # Input DTO
│   │   │   ├── result.rs     # Output DTO
│   │   │   ├── handler.rs    # Use case logic
│   │   │   └── mod.rs
│   │   ├── update_anime_score/
│   │   ├── discover_relations/
│   │   └── search_anime/     # Query handler (CQRS query)
│   ├── ports/                # Interfaces for infrastructure
│   │   ├── anime_repository.rs
│   │   ├── provider_client.rs
│   │   ├── event_publisher.rs
│   │   └── mod.rs
│   └── service.rs            # Application service (orchestration)
├── infrastructure/
│   ├── persistence/
│   │   ├── repositories/     # Repository implementations
│   │   │   ├── anime_repository_impl.rs
│   │   │   ├── anime_relations_repository_impl.rs
│   │   │   ├── anime_query_repository_impl.rs
│   │   │   └── mod.rs
│   │   ├── mapper.rs         # Entity ↔ Model conversions
│   │   └── mod.rs
│   ├── adapters/             # External service adapters
│   └── models.rs             # Database models (Diesel)
├── commands/                 # Tauri commands (presentation)
│   ├── progressive_relations.rs
│   └── auto_enrichment.rs
├── commands.rs               # Command handlers
└── mod.rs
```

---

## Design Patterns

### 1. **Repository Pattern**

**Purpose**: Abstract data access behind clean interfaces.

**Implementation**:
- Application layer defines `trait AnimeRepository`
- Infrastructure layer implements `struct AnimeRepositoryImpl`
- Domain layer never knows about databases

**Benefits**:
- Easy to mock for testing
- Can swap database implementations
- Clean separation of concerns

**Example**:
```rust
// Application defines the interface
#[async_trait]
pub trait AnimeRepository: Send + Sync {
    async fn save(&self, aggregate: &AnimeAggregate) -> AppResult<()>;
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<AnimeDetailed>>;
}

// Infrastructure implements it
pub struct AnimeRepositoryImpl {
    db: Arc<Database>,
}

#[async_trait]
impl AnimeRepository for AnimeRepositoryImpl {
    async fn save(&self, aggregate: &AnimeAggregate) -> AppResult<()> {
        // Diesel database code here
    }
}
```

---

### 2. **Aggregate Pattern (DDD)**

**Purpose**: Define consistency boundaries and encapsulate business logic.

**Implementation**:
- `AnimeAggregate` wraps `AnimeDetailed` entity
- All modifications go through aggregate methods
- Aggregate publishes domain events

**Benefits**:
- Business rules enforced in one place
- Consistency guaranteed within boundary
- Event sourcing ready

**Example**:
```rust
pub struct AnimeAggregate {
    entity: AnimeDetailed,
    pending_events: Vec<Box<dyn DomainEvent>>,
}

impl AnimeAggregate {
    pub fn update_score(&mut self, new_score: f32) -> Result<(), String> {
        // Validate business rule
        self.entity.update_score(new_score)?;
        
        // Publish event
        self.pending_events.push(Box::new(
            AnimeScoreUpdatedEvent::new(self.entity.id, old_score, new_score)
        ));
        
        Ok(())
    }
}
```

---

### 3. **CQRS (Command Query Responsibility Segregation)**

**Purpose**: Separate read and write operations for clarity and optimization.

**Implementation**:
- **Commands**: Modify state (CreateAnime, UpdateScore)
- **Queries**: Read data (SearchAnime, GetAnimeById)

**Benefits**:
- Clear intent (is this changing data or reading?)
- Different optimization strategies
- Easy to scale reads and writes independently

**Example**:
```rust
// Command (write)
#[async_trait]
pub trait UseCase<TCommand, TResult> {
    async fn execute(&self, command: TCommand) -> AppResult<TResult>;
}

// Query (read)
#[async_trait]
pub trait Query<TQuery, TResult> {
    async fn execute(&self, query: TQuery) -> AppResult<TResult>;
}

// Usage
impl UseCase<UpdateAnimeScoreCommand, UpdateAnimeScoreResult> for UpdateAnimeScoreHandler {
    async fn execute(&self, command: UpdateAnimeScoreCommand) -> AppResult<UpdateAnimeScoreResult> {
        // Modify state
    }
}

impl Query<SearchAnimeQuery, SearchAnimeResult> for SearchAnimeHandler {
    async fn execute(&self, query: SearchAnimeQuery) -> AppResult<SearchAnimeResult> {
        // Read data
    }
}
```

---

### 4. **Specification Pattern**

**Purpose**: Build complex queries in a composable, reusable way.

**Implementation**:
- `AnimeSearchSpecification` defines query criteria
- Repository implements `find_by_criteria(spec)`

**Benefits**:
- Queries are type-safe and composable
- Business logic stays in domain layer
- Easy to test query logic

**Example**:
```rust
pub struct AnimeSearchSpecification {
    pub title_contains: Option<String>,
    pub min_score: Option<f32>,
    pub max_score: Option<f32>,
    pub providers: Option<Vec<AnimeProvider>>,
    pub genres: Option<Vec<String>>,
    pub year: Option<i32>,
}

// Usage
let spec = AnimeSearchSpecification {
    min_score: Some(8.0),
    genres: Some(vec!["Thriller".to_string()]),
    year: Some(2018),
    ..Default::default()
};

let results = query_repo.find_by_criteria(spec, pagination).await?;
```

---

### 5. **Event Sourcing**

**Purpose**: Track all state changes as events for auditing and debugging.

**Implementation**:
- Aggregates publish domain events
- Events stored for history/replay
- Event handlers can trigger side effects

**Benefits**:
- Complete audit trail
- Can rebuild state from events
- Easy to add new features (just listen to events)

**Example**:
```rust
pub trait DomainEvent: Send + Sync {
    fn occurred_at(&self) -> DateTime<Utc>;
    fn event_id(&self) -> Uuid;
    fn event_type(&self) -> &'static str;
}

pub struct AnimeScoreUpdatedEvent {
    pub event_id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub anime_id: Uuid,
    pub old_score: Option<f32>,
    pub new_score: f32,
}
```

---

### 6. **Ports & Adapters (Hexagonal Architecture)**

**Purpose**: Isolate business logic from external concerns.

**Implementation**:
- **Ports**: Interfaces defined by application (e.g., `AnimeRepository`)
- **Adapters**: Implementations in infrastructure (e.g., `AnimeRepositoryImpl`)

**Benefits**:
- Business logic independent of frameworks
- Easy to swap implementations (PostgreSQL → MongoDB)
- Testable without external dependencies

**Diagram**:
```
┌─────────────────────────────────────────┐
│         Application Layer               │
│  ┌─────────────────────────────────┐   │
│  │   Use Cases & Domain Logic       │   │
│  └─────────────────────────────────┘   │
│              ↓ uses ↓                   │
│  ┌─────────────────────────────────┐   │
│  │   Ports (Interfaces)             │   │
│  │   - AnimeRepository              │   │
│  │   - ProviderClient               │   │
│  │   - EventPublisher               │   │
│  └─────────────────────────────────┘   │
└─────────────────────────────────────────┘
              ↓ implemented by ↓
┌─────────────────────────────────────────┐
│      Infrastructure Layer               │
│  ┌─────────────────────────────────┐   │
│  │   Adapters (Implementations)     │   │
│  │   - AnimeRepositoryImpl          │   │
│  │   - JikanClient                  │   │
│  │   - AniListClient                │   │
│  └─────────────────────────────────┘   │
└─────────────────────────────────────────┘
```

---

## Data Flow

### **Write Operation (Command)**

```
User Action (Frontend)
    ↓
Tauri Command (Presentation)
    ↓
Use Case Handler (Application)
    ↓
Domain Aggregate (Domain)
    ↓ publishes events
Domain Events
    ↓
Repository (Infrastructure)
    ↓
Database (PostgreSQL)
```

**Example**: Update anime score

```rust
// 1. User clicks "Update Score" → Frontend calls Tauri command
#[tauri::command]
pub async fn update_anime_score(
    request: UpdateScoreRequest,
    handler: State<'_, Arc<UpdateAnimeScoreHandler>>,
) -> Result<UpdateAnimeScoreResult, String> {
    // 2. Delegate to use case handler
    let command = UpdateAnimeScoreCommand {
        anime_id: request.anime_id,
        new_score: request.new_score,
    };
    
    handler.execute(command).await.map_err(|e| e.to_string())
}

// 3. Use case handler orchestrates
impl UseCase<UpdateAnimeScoreCommand, UpdateAnimeScoreResult> {
    async fn execute(&self, command: UpdateAnimeScoreCommand) -> AppResult<UpdateAnimeScoreResult> {
        // Load from repository
        let anime = self.anime_repository.find_by_id(command.anime_id).await?;
        
        // 4. Execute domain logic (aggregate)
        let mut aggregate = AnimeAggregate::from_entity(anime);
        aggregate.update_score(command.new_score)?;  // Business rules enforced
        
        // 5. Persist
        self.anime_repository.update(&aggregate).await?;
        
        // 6. Publish events
        let events = aggregate.pending_events();
        self.event_publisher.publish_all(events).await?;
        
        Ok(UpdateAnimeScoreResult::new(command.anime_id, command.new_score))
    }
}
```

---

### **Read Operation (Query)**

```
User Action (Frontend)
    ↓
Tauri Command (Presentation)
    ↓
Query Handler (Application)
    ↓
Repository (Infrastructure)
    ↓
Database (PostgreSQL)
    ↓
Domain Entity
    ↓
Frontend (DTO)
```

**Example**: Search anime

```rust
// 1. User types search query → Frontend calls Tauri command
#[tauri::command]
pub async fn search_anime(
    request: SearchRequest,
    handler: State<'_, Arc<SearchAnimeHandler>>,
) -> Result<SearchAnimeResult, String> {
    // 2. Delegate to query handler
    let query = SearchAnimeQuery {
        search_term: request.query,
        pagination: request.pagination,
    };
    
    handler.execute(query).await.map_err(|e| e.to_string())
}

// 3. Query handler reads from repository
impl Query<SearchAnimeQuery, SearchAnimeResult> {
    async fn execute(&self, query: SearchAnimeQuery) -> AppResult<SearchAnimeResult> {
        // 4. Repository queries database
        self.anime_repository
            .search(&query.search_term, query.pagination)
            .await
    }
}
```

---

## Repository Pattern

### **Repository Interfaces (Ports)**

Defined in **application layer** (`modules/anime/application/ports/`):

```rust
#[async_trait]
pub trait AnimeRepository: Send + Sync {
    async fn save(&self, aggregate: &AnimeAggregate) -> AppResult<()>;
    async fn update(&self, aggregate: &AnimeAggregate) -> AppResult<()>;
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<AnimeDetailed>>;
    async fn delete(&self, id: Uuid) -> AppResult<()>;
    async fn search(&self, query: &str, pagination: PaginationParams) -> AppResult<PaginatedResult<AnimeDetailed>>;
}

#[async_trait]
pub trait AnimeRelationsRepository: Send + Sync {
    async fn save_relations(&self, anime_id: Uuid, relations: Vec<AnimeRelation>) -> AppResult<()>;
    async fn find_relations(&self, anime_id: Uuid) -> AppResult<Vec<AnimeRelation>>;
    async fn find_bidirectional_relations(&self, anime_id: Uuid) -> AppResult<Vec<AnimeRelation>>;
}

#[async_trait]
pub trait AnimeQueryRepository: Send + Sync {
    async fn find_by_criteria(&self, spec: AnimeSearchSpecification, pagination: PaginationParams) -> AppResult<PaginatedResult<AnimeDetailed>>;
    async fn count_by_criteria(&self, spec: AnimeSearchSpecification) -> AppResult<u64>;
}
```

### **Repository Implementations (Adapters)**

Implemented in **infrastructure layer** (`modules/anime/infrastructure/persistence/repositories/`):

```rust
// Core CRUD operations
pub struct AnimeRepositoryImpl {
    db: Arc<Database>,
}

// Relations-specific operations
pub struct AnimeRelationsRepositoryImpl {
    db: Arc<Database>,
}

// Complex queries and search
pub struct AnimeQueryRepositoryImpl {
    db: Arc<Database>,
}
```

### **Repository Split Rationale**

The original 1,870-line monolithic repository was split into **3 focused repositories**:

1. **AnimeRepositoryImpl** (Core CRUD)
   - Save, update, delete anime
   - Find by ID, external ID
   - Batch operations
   - External ID management

2. **AnimeRelationsRepositoryImpl** (Relations)
   - Save/delete relations
   - Bidirectional relation creation
   - Load anime with relations
   - Batch relation loading

3. **AnimeQueryRepositoryImpl** (Complex Queries)
   - Specification pattern queries
   - Title variations search
   - Genre/studio filtering
   - Fuzzy search with ranking

**Benefits**:
- Single Responsibility Principle
- Easier to test and mock
- Smaller, more maintainable files
- Clear separation of concerns

---

## Domain Events

### **Event Types**

Defined in `modules/anime/domain/events/anime_events.rs`:

```rust
pub trait DomainEvent: Send + Sync {
    fn occurred_at(&self) -> DateTime<Utc>;
    fn event_id(&self) -> Uuid;
    fn event_type(&self) -> &'static str;
}

pub struct AnimeCreatedEvent {
    pub event_id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub anime_id: Uuid,
    pub title: String,
    pub provider: String,
    pub external_id: String,
}

pub struct AnimeScoreUpdatedEvent {
    pub event_id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub anime_id: Uuid,
    pub old_score: Option<f32>,
    pub new_score: f32,
}

pub struct RelationsDiscoveredEvent {
    pub event_id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub anime_id: Uuid,
    pub relations_count: usize,
    pub source: String,
}

pub struct AnimeEnrichedEvent {
    pub event_id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub anime_id: Uuid,
    pub enrichment_source: String,
    pub fields_updated: Vec<String>,
}
```

### **Event Publishing**

```rust
// Aggregate publishes events
impl AnimeAggregate {
    pub fn update_score(&mut self, new_score: f32) -> Result<(), String> {
        let old_score = self.entity.score;
        self.entity.update_score(new_score)?;
        
        // Publish event
        let event = AnimeScoreUpdatedEvent::new(self.entity.id, old_score, new_score);
        self.pending_events.push(Box::new(event));
        
        Ok(())
    }
    
    pub fn pending_events(&self) -> &[Box<dyn DomainEvent>] {
        &self.pending_events
    }
}

// Use case handler publishes to event publisher
impl UseCase<UpdateAnimeScoreCommand, UpdateAnimeScoreResult> {
    async fn execute(&self, command: UpdateAnimeScoreCommand) -> AppResult<UpdateAnimeScoreResult> {
        let mut aggregate = /* ... */;
        aggregate.update_score(command.new_score)?;
        
        // Publish events
        let (_, events) = aggregate.into_parts();
        self.event_publisher.publish_all(events).await?;
        
        Ok(/* ... */)
    }
}
```

---

## Testing Strategy

### **Unit Tests**

Test business logic in isolation without external dependencies.

**Location**: Same file as implementation (`#[cfg(test)] mod tests`)

**Example**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_score_validates_range() {
        let mut aggregate = AnimeAggregate::create(
            AnimeProvider::AniList,
            "12345".to_string(),
            "Death Note".to_string(),
        );
        
        // Valid score
        assert!(aggregate.update_score(8.5).is_ok());
        
        // Invalid score (too high)
        assert!(aggregate.update_score(11.0).is_err());
        
        // Invalid score (negative)
        assert!(aggregate.update_score(-1.0).is_err());
    }
    
    #[test]
    fn test_update_score_publishes_event() {
        let mut aggregate = /* ... */;
        aggregate.clear_events();
        
        aggregate.update_score(8.5).unwrap();
        
        assert_eq!(aggregate.pending_events().len(), 1);
        assert_eq!(aggregate.pending_events()[0].event_type(), "AnimeScoreUpdated");
    }
}
```

---

### **Integration Tests**

Test multiple layers working together with real database.

**Location**: `tests/` directory

**Example**:
```rust
// tests/anime_relations_integration_test.rs
#[tokio::test]
async fn test_bidirectional_relations_created_automatically() {
    let test_db = TestDb::new();
    let services = build_test_services(&test_db);
    
    // Create two anime
    let anime_a = create_test_anime("Steins;Gate", &services).await;
    let anime_b = create_test_anime("Steins;Gate 0", &services).await;
    
    // Save relation A → B (sequel)
    services.relations_repository
        .save_relations(anime_a.id, vec![
            AnimeRelation {
                related_anime_id: anime_b.id,
                relation_type: AnimeRelationType::Sequel,
                related_title: Some("Steins;Gate 0".to_string()),
            }
        ])
        .await
        .unwrap();
    
    // Verify bidirectional relation B → A (prequel) was auto-created
    let inverse_relations = services.relations_repository
        .find_relations(anime_b.id)
        .await
        .unwrap();
    
    assert_eq!(inverse_relations.len(), 1);
    assert_eq!(inverse_relations[0].related_anime_id, anime_a.id);
    assert_eq!(inverse_relations[0].relation_type, AnimeRelationType::Prequel);
}
```

---

### **Test Utilities**

**Location**: `tests/utils/`

- `test_db.rs`: Isolated test database for each test
- `helpers.rs`: Build test services with mocked dependencies
- `factories.rs`: Create test data easily

**Example**:
```rust
// tests/utils/helpers.rs
pub fn build_test_services(test_db: &TestDb) -> TestServices {
    let db = test_db.get_database();
    let anime_repository = Arc::new(AnimeRepositoryImpl::new(db.clone()));
    let relations_repository = Arc::new(AnimeRelationsRepositoryImpl::new(db.clone()));
    
    TestServices {
        anime_repository,
        relations_repository,
        // ...
    }
}
```

---

## Development Guidelines

### **1. Adding a New Use Case**

**Steps**:

1. **Create use case folder**: `modules/anime/application/use_cases/your_use_case/`

2. **Define command** (`command.rs`):
```rust
#[derive(Debug, Clone)]
pub struct YourCommand {
    pub field1: String,
    pub field2: i32,
}
```

3. **Define result** (`result.rs`):
```rust
#[derive(Debug, Clone)]
pub struct YourResult {
    pub success: bool,
    pub data: String,
}
```

4. **Implement handler** (`handler.rs`):
```rust
pub struct YourHandler {
    repository: Arc<dyn AnimeRepository>,
}

#[async_trait]
impl UseCase<YourCommand, YourResult> for YourHandler {
    async fn execute(&self, command: YourCommand) -> AppResult<YourResult> {
        // Implementation
    }
}
```

5. **Export from mod.rs**

6. **Add Tauri command** (if needed)

7. **Write tests**

---

### **2. Adding a New Repository Method**

**Steps**:

1. **Add method to port** (`application/ports/anime_repository.rs`):
```rust
#[async_trait]
pub trait AnimeRepository: Send + Sync {
    async fn your_new_method(&self, param: String) -> AppResult<Vec<AnimeDetailed>>;
}
```

2. **Implement in repository** (`infrastructure/persistence/repositories/anime_repository_impl.rs`):
```rust
#[async_trait]
impl AnimeRepository for AnimeRepositoryImpl {
    async fn your_new_method(&self, param: String) -> AppResult<Vec<AnimeDetailed>> {
        // Implementation
    }
}
```

3. **Write tests**

---

### **3. Adding a Domain Event**

**Steps**:

1. **Define event** (`domain/events/anime_events.rs`):
```rust
pub struct YourNewEvent {
    pub event_id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub anime_id: Uuid,
    pub custom_field: String,
}

impl YourNewEvent {
    pub fn new(anime_id: Uuid, custom_field: String) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            anime_id,
            custom_field,
        }
    }
}

impl DomainEvent for YourNewEvent {
    fn occurred_at(&self) -> DateTime<Utc> { self.occurred_at }
    fn event_id(&self) -> Uuid { self.event_id }
    fn event_type(&self) -> &'static str { "YourNewEvent" }
}
```

2. **Publish from aggregate**:
```rust
impl AnimeAggregate {
    pub fn your_action(&mut self) {
        // Business logic
        
        // Publish event
        let event = YourNewEvent::new(self.entity.id, "data".to_string());
        self.pending_events.push(Box::new(event));
    }
}
```

---

### **4. Code Quality Checklist**

Before committing:

- [ ] **No business logic in infrastructure** (repositories should be thin)
- [ ] **No database code in domain** (keep domain pure)
- [ ] **All public methods documented** (rustdoc comments)
- [ ] **Unit tests for business logic** (domain layer)
- [ ] **Integration tests for critical paths** (cross-layer)
- [ ] **Error handling with AppError** (no unwrap/expect in production code)
- [ ] **Async functions use tokio::spawn_blocking** for blocking operations
- [ ] **Transactions for multi-step operations**
- [ ] **Logging at appropriate levels** (debug, info, warn, error)

---

### **5. Error Handling**

**Use AppError enum** (`shared/errors/app_error.rs`):

```rust
pub enum AppError {
    DatabaseError(String),
    NotFound(String),
    ValidationError(String),
    InvalidInput(String),
    // ...
}

pub type AppResult<T> = Result<T, AppError>;
```

**Example**:
```rust
async fn find_by_id(&self, id: Uuid) -> AppResult<Option<AnimeDetailed>> {
    let anime = anime::table
        .find(id)
        .first::<Anime>(conn)
        .optional()
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
    
    Ok(anime.map(|a| mapper::model_to_entity(a, /* ... */)))
}
```

---

### **6. Logging Best Practices**

```rust
use crate::{log_debug, log_error, log_info, log_warn};

// Debug: Detailed diagnostic info
log_debug!("Loading anime with ID: {}", anime_id);

// Info: Important state changes
log_info!("Successfully saved anime: {}", anime.title.main);

// Warn: Recoverable errors
log_warn!("Anime not found with ID {}, creating new", anime_id);

// Error: Unrecoverable errors
log_error!("Failed to connect to database: {}", err);
```

---

## Summary

Miru's backend architecture follows **industry-standard patterns** to ensure:

✅ **Maintainability**: Small, focused files with clear responsibilities  
✅ **Testability**: Easy to mock dependencies and test in isolation  
✅ **Scalability**: Can add new features without modifying existing code  
✅ **Type Safety**: Compile-time guarantees via Rust's type system  
✅ **Clean Code**: No duplication, clear separation of concerns  
✅ **Documentation**: Code structure self-documents the architecture  

**Key Patterns**:
- **DDD**: Aggregates, entities, value objects, domain events
- **Clean Architecture**: Layer separation with dependency inversion
- **Hexagonal Architecture**: Ports (interfaces) and adapters (implementations)
- **CQRS**: Commands and queries separated
- **Event Sourcing**: All state changes tracked as events
- **Repository Pattern**: Data access abstracted behind clean interfaces
- **Specification Pattern**: Complex queries composable and reusable

**For Questions or Contributions**:
- Follow the guidelines in this document
- Write tests for all new code
- Keep business logic in domain layer
- Use ports (interfaces) for infrastructure dependencies

---

**Last Updated**: 2025-01-18  
**Architecture Version**: 2.0 (Post-DDD Restructuring)
