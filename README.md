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

## Getting Started

### Prerequisites
- Rust (latest stable version)
- Node.js (v16 or higher)
- npm or yarn

### Installation
1. Install backend dependencies:
   ```bash
   cd backend
   cargo build
   ```

2. Install frontend dependencies:
   ```bash
   cd frontend
   npm install
   ```

### Development

Run both servers with the development script:
```bash
./dev.sh
```

Or run them separately:

**Backend:**
```bash
cd backend
cargo run
```

**Frontend:**
```bash
cd frontend
npm run dev
```

The backend runs on http://localhost:3000
The frontend runs on http://localhost:5173

## Features

- ✅ Interactive Rust code editor with syntax highlighting
- ✅ Real-time code execution via Rust Playground API
- ✅ Dark theme optimized for coding
- 🚧 Course content and lessons (coming soon)
- 🚧 User progress tracking (coming soon)