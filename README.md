# Rust & Solana Learning Platform

An interactive online platform for learning Rust programming language and Solana blockchain development.

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

- ✅ Interactive code editor with syntax highlighting (Monaco Editor)
- ✅ Real-time Rust code execution via Rust Playground API
- ✅ Structured learning paths for Rust and Solana
- ✅ Course navigation with chapters and lessons
- ✅ Built-in hints and solutions
- ✅ Dark theme optimized for coding
- ✅ Responsive design
- 🚧 User authentication and progress tracking (coming soon)
- 🚧 More advanced Solana examples (coming soon)

## Courses Available

### 🦀 Rust Programming
- Getting Started
  - Hello, World!
  - Variables and Mutability
- Ownership
  - Ownership Basics
  - Borrowing and References

### ⚡ Solana Development
- Introduction to Solana
  - What is Solana
  - Connecting to the Network