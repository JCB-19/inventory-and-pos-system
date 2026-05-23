# Inventory with POS System

A full-stack Inventory and Point-of-Sale (POS) system built using **Rust (Actix Web)** for the backend and **Next.js** for the frontend. This project demonstrates authentication, role-based access control, inventory management, and transaction handling with a PostgreSQL database.

---

## 🚀 Tech Stack

### Backend
- Rust
- Actix Web
- SQLx

### Frontend
- Next.js
- TypeScript
- Tailwind CSS

### Database
- PostgreSQL

---

## ✨ Features

- JWT-based authentication
- Role-based access control (Admin / Manager / POS Staff)
- Product inventory management
- POS transaction system
- Branch/company structure support
- REST API backend built with Rust
- Responsive UI with Next.js

---

## 📁 Project Structure
Inventory_with_POS_system/
├── backend/ # Rust (Actix Web API)
│ ├── src/
│ │ ├── middleware/
│ │ ├── routes/
│ │ ├── main.rs
│ │ └── middleware.rs
│ │ ├── routes.rs
│ │ └── state.rs
│ │
│ ├── .gitignore
│ └── Cargo.toml
│
├── frontend/ # Next.js (App Router)
│ ├── src/
│ │ └── app/
│ │ ├── components/
│ │ ├── dashboard/
│ │ ├── inventory/
│ │ ├── lib/
│ │ ├── pos/
│ │ ├── globals.css
│ │ ├── layout.tsx
│ │ └── page.tsx
│ │
│ ├── public/
│ ├── package.json
│ ├──  next.config.js
│ └── .gitignore
|
└── README.md
