# Rust Book Online

An interactive online platform for learning Rust programming language.

## Architecture

- **Backend**: Rust + Axum web framework
- **Frontend**: React + TypeScript + Vite
- **Code Editor**: Monaco Editor
- **Code Execution**: Rust Playground API / Docker containers

## Project Structure

```
rust_book_online/
├── backend/          # Rust backend API
│   ├── src/
│   └── Cargo.toml
├── frontend/         # React frontend
│   ├── src/
│   └── package.json
└── shared/          # Shared types/utilities
```

## Development

### Backend
```bash
cd backend
cargo run
```

### Frontend
```bash
cd frontend
npm run dev
```

The backend runs on http://localhost:3000
The frontend runs on http://localhost:5173