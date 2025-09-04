# Miru - Anime Collection Manager

A modern anime collection management application built with Tauri, React, and Rust, following Domain-Driven Design (DDD) and Clean Architecture principles.

## Architecture Overview

The application follows a clean architecture with clear separation of concerns:

- **Domain Layer**: Core business logic and entities
- **Application Layer**: Use cases and application services
- **Infrastructure Layer**: External services, database, and caching
- **Presentation Layer**: React frontend with TypeScript

## Prerequisites

- Rust (latest stable version)
- Node.js (v18 or higher) and pnpm
- PostgreSQL (v14 or higher)
- Redis (v6 or higher)
- Tauri CLI

## Setup Instructions

### 1. Database Setup

```bash
# Create a PostgreSQL database
createdb miru_db

# Update the DATABASE_URL in .env file
DATABASE_URL=postgres://username:password@localhost:5432/miru_db
```

### 2. Redis Setup

```bash
# Start Redis server
redis-server

# Or using Docker
docker run -d -p 6379:6379 redis:latest
```

### 3. Environment Configuration

```bash
# Copy the example environment file
cp .env.example .env

# Edit .env with your configuration
DATABASE_URL=postgres://your_user:your_password@localhost:5432/miru_db
REDIS_URL=redis://127.0.0.1:6379
RUST_LOG=info
```

### 4. Install Dependencies

```bash
# Install Rust dependencies
cd src-tauri
cargo build

# Install frontend dependencies
cd ..
pnpm install

# Install Tauri CLI if not already installed
cargo install tauri-cli
```

### 5. Run Database Migrations

The migrations will run automatically when the application starts, but you can also run them manually:

```bash
cd src-tauri
diesel migration run
```

### 6. Development

```bash
# Run the development server
pnpm tauri dev

# Or run frontend and backend separately
# Terminal 1: Frontend
pnpm dev

# Terminal 2: Backend
cd src-tauri
cargo run
```

### 7. Building for Production

```bash
# Build the application
pnpm tauri build

# The built application will be in src-tauri/target/release
```

## Project Structure

```
miru/
├── src/                    # React frontend
│   ├── features/          # Feature modules
│   │   └── anime/        # Anime feature
│   │       ├── domain/   # Domain models
│   │       ├── hooks/    # React hooks
│   │       └── infrastructure/
│   ├── components/        # React components
│   └── pages/            # Application pages
│
└── src-tauri/            # Rust backend
    ├── src/
    │   ├── domain/       # Domain layer
    │   │   ├── entities/
    │   │   ├── events/
    │   │   ├── repositories/
    │   │   ├── services/
    │   │   └── value_objects/
    │   │
    │   ├── application/  # Application layer
    │   │   ├── commands/ # Tauri commands
    │   │   └── services/ # Application services
    │   │
    │   ├── infrastructure/ # Infrastructure layer
    │   │   ├── cache/      # Redis cache
    │   │   ├── database/   # PostgreSQL
    │   │   └── external/   # External APIs (Jikan)
    │   │
    │   └── shared/         # Shared utilities
    │       ├── errors/
    │       └── utils/
    │
    └── migrations/         # Database migrations
```

## Features

### Core Features
- ✅ Anime search and discovery (via Jikan API)
- ✅ Collection management
- ✅ Advanced scoring system with quality metrics
- ✅ Bulk import from CSV/text files
- ✅ Seasonal anime tracking
- ✅ Redis caching for performance
- ✅ PostgreSQL for persistent storage

### Scoring System
The application uses a sophisticated scoring algorithm that considers:
- Bayesian rating (to handle small sample sizes)
- Popularity metrics
- Audience reach
- Engagement scores
- Momentum scoring for new shows
- Favorites intensity

### API Integration
- **Jikan API**: MyAnimeList unofficial API for anime data
- Rate limiting and retry logic implemented
- Fallback to cached data when API is unavailable

## API Endpoints (Tauri Commands)

### Anime Commands
- `search_anime` - Search for anime by title
- `get_anime_by_id` - Get anime by UUID
- `get_anime_by_mal_id` - Get anime by MyAnimeList ID
- `update_anime` - Update anime information
- `delete_anime` - Delete anime from database
- `get_top_anime` - Get top-rated anime
- `get_seasonal_anime` - Get anime by season
- `recalculate_scores` - Recalculate anime scores
- `get_recommendations` - Get anime recommendations

### Collection Commands
- `create_collection` - Create new collection
- `get_collection` - Get collection by ID
- `get_all_collections` - Get all collections
- `update_collection` - Update collection details
- `delete_collection` - Delete collection
- `add_anime_to_collection` - Add anime to collection
- `remove_anime_from_collection` - Remove anime from collection
- `get_collection_anime` - Get all anime in collection
- `update_anime_in_collection` - Update user score/notes
- `get_collection_statistics` - Get collection statistics

### Import Commands
- `import_anime_batch` - Import multiple anime by titles
- `import_from_mal_ids` - Import by MAL IDs
- `import_from_csv` - Import from CSV file
- `import_seasonal` - Import entire season

## Performance Optimizations

1. **Caching Strategy**
   - Redis caching for frequently accessed data
   - Cache invalidation on updates
   - TTL-based expiration

2. **Database Optimizations**
   - Indexed columns for fast queries
   - Batch operations for bulk imports
   - Connection pooling with r2d2

3. **Rate Limiting**
   - Respects Jikan API rate limits (1 req/sec)
   - Exponential backoff for retries
   - Request queuing

## Contributing

1. Follow the DDD and Clean Architecture principles
2. Write tests for new features
3. Update documentation
4. Follow Rust best practices and clippy recommendations

## License

MIT

## Troubleshooting

### Database Connection Issues
- Ensure PostgreSQL is running
- Check DATABASE_URL in .env file
- Verify user permissions

### Redis Connection Issues
- Ensure Redis server is running
- Check REDIS_URL in .env file
- Default port is 6379

### API Rate Limiting
- The app respects Jikan's rate limits
- If you encounter 429 errors, the app will automatically retry
- Consider using caching more aggressively

### Build Issues
- Clear cargo cache: `cargo clean`
- Clear node_modules: `rm -rf node_modules && pnpm install`
- Ensure all prerequisites are installed
