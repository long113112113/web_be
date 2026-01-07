# Rust Axum Backend - IronClad Edition 🦀

This project is a high-performance, safe, and structured microservice backend built with **Rust**, **Axum**, and **SQLx**. It follows a strictly layered architecture to ensure scalability and maintainability.

## 🏗️ Architecture

The project is organized into distinct layers:

1.  **Handlers (`src/handlers/`)**: The entry point for HTTP requests. Handles request parsing and response formatting.
2.  **Services (`src/services/`)**: Contains the business logic. It orchestrates data flow between handlers and repositories.
3.  **Repositories (`src/repository/`)**: Direct data access layer. Executes SQL queries using `sqlx`.
4.  **Models (`src/models/`)**: Domain entities reflecting the database schema.
5.  **DTOs (`src/dtos/`)**: Data Transfer Objects for type-safe API communication.

```mermaid
graph TD
    Client[Client] -->|HTTP Request| Router[Axum Router]
    Router -->|Route| Handler[Handler Layer]
    Handler -->|DTO| Service[Service Layer]
    Service -->|Domain Model| Repository[Repository Layer]
    Repository -->|SQL| DB[(PostgreSQL)]
```

## 🚀 Getting Started

### Prerequisites

- Rust (latest stable)
- PostgreSQL
- `sqlx-cli` (`cargo install sqlx-cli`)

### Environment Variables

Create a `.env` file in the root directory with the following variables:

```env
DATABASE_URL=postgres://user:password@localhost:5432/dbname
JWT_SECRET=your_super_secret_jwt_key
CORS_ORIGINS=http://localhost:3000,http://example.com
```

### Running the Project

1.  **Database Setup**:
    ```bash
    sqlx database create
    sqlx migrate run
    ```

2.  **Start the Server**:
    ```bash
    cargo run
    ```

## 📚 API Summary

### Authentication

-   `POST /api/auth/register`: Register a new user.
-   `POST /api/auth/login`: Login and receive JWTs.
-   `POST /api/auth/refresh`: Refresh access token using a refresh token.

## 🛡️ Key Features

-   **Zero-Panic Policy**: Robust error handling using `Result` and custom error types.
-   **Type-Safe SQL**: Compile-time checked SQL queries with `sqlx`.
-   **Secure Authentication**: Argon2 password hashing and JWT-based auth.
-   **Structured Layout**: Clean separation of concerns for easy testing and updates.

---

*Refactored by IronClad 🦀*
