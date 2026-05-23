// ─── Types ───────────────────────────────────────────────────────────────────
// These interfaces define the shape of data the frontend sends and receives.
// They act as a contract between the frontend and the backend API.

// Represents a single product in the inventory.
// id is optional because it doesn't exist yet when creating a new product.
export interface Product {
  id?: number;
  name: string;
  sku: string;
  category?: string; // Optional — not all products need a category
  price: number;
  stock: number;
  branch_name?: string;
}

// Represents a single line item inside an order.
// Stores the price at the time of purchase so historical records stay accurate
// even if the product price changes later.
export interface SoldItem {
  product_id: number;
  quantity: number;
  price: number; // Price per unit at the time of the sale
}

// Represents a completed POS transaction.
// id and createdAt are optional because they are assigned by the backend after saving.
export interface SoldItemInfo {
  id?: number;
  items: SoldItem[];
  total: number;       // Grand total including VAT
  amount_paid: number;  // Cash handed over by the customer
  change: number;
}

// Represents all the analytics data shown on the Dashboard page.
// The backend computes and aggregates all of this in a single API call.
export interface DashboardSummary {
  total_revenue: number;    // Sum of all order totals
  total_quantity_sold: number;       // Total number of individual items sold
  total_sold: number;     // Total number of completed transactions
  avg_order_value: number;   // total_revenue divided by total_orders
  // Top 5 products ranked by quantity sold
  top_products: { name: string; quantity: number; revenue: number }[];
  // Last 10 orders, newest first — shown in the Recent Transactions table
  recent_transactions: {
    id: number;
    created_at: string;   // ISO timestamp string from the backend
    items: number;  // Number of distinct items in the order
    total: number;
  }[];
  // Products with stock at or below 10 — shown in the Low Stock Alerts panel
  low_stock: { name: string; stock: number }[];
}

export type AddProductResponse = {
  message: string;
};

// ─── Auth Types ───────────────────────────────────────────────────────────────

// Shape of the response returned by /Login and /Register
export interface AuthResponse {
  message: string;
  token?: string;  // JWT token returned on successful login
  role?: string;   // User role returned on successful login (e.g. "admin", "cashier")
  username?: string; // Username returned on successful register
}

//================================API ENDPOINTS==================================

const PRODUCTS = "/product";
const AUTH_BASE = "/auth";
const POS = "/pos";
const DASHBOARD = "/main";

//===============================================================================

//===========================PRODUCT REQUESTS FUNCTIONS==========================

// A generic fetch wrapper used by every API function below.
// T is the expected return type so callers get full TypeScript type safety.
// - Automatically sets Content-Type to JSON on every request
// - Attaches the JWT token from localStorage to every request as an Authorization header
// - Throws a descriptive error if the response is not OK (non-2xx status)
//   so the calling component can catch it and handle it appropriately
async function request<T>(
  path: string,         // API path appended to PRODUCTS (e.g. "/products")
  options?: RequestInit // Optional fetch options: method, body, headers, etc.
): Promise<T> {

  // Read the token from localStorage — set after a successful login
  const token = localStorage.getItem("token");

  const res = await fetch(`${PRODUCTS}${path}`, {
    method: options?.method ?? "GET",
    headers: {
      // Attach token if it exists — backend uses this to verify the request is authenticated
      ...(token ? { Authorization: `Bearer ${token}` } : {}),
      ...(options?.body ? { "Content-Type": "application/json" } : {}), // Only set Content-Type if there's a body to avoid issues with GET requests
      ...(options?.headers ?? {}), // Allow callers to add custom headers if needed
    },
    body: options?.body, // Body should already be stringified by the caller (e.g. JSON.stringify(payload))
  });

  let errText = "";

  if (!res.ok) {
    try {
      const data = await res.clone().json();
      errText = data.error;
    } catch {
      errText = await res.text();
    }

    console.error("Backend error:", res.status, res.url, errText || "No message");
    throw new Error(errText || `Request failed: ${res.status}`);
  }

  // only runs when res.ok === true
  return res.json();
}

// CRUD functions for the Inventory page and POS product grid.

// Fetch the full list of products — used by both Inventory and POS pages on load
// Coerces price from string to number since the backend stores it as a string
export const getProducts = async () => {
  const products = await request<Product[]>("/view_product");

  return products.map((p) => ({
    ...p,
    price: Number(p.price), // Convert price from string → number for frontend use
  }));
};

// Create a new product — Omit<Product, "id"> ensures id is never sent
// since the backend generates it
export const addProduct = async (product: Omit<Product, "id">) => {
    // Convert price from number -> string for backend
    const payload = {
      ...product,
      price: product.price.toString(),
    };

    return request<AddProductResponse>("/add_product", {
      method: "POST",
      body: JSON.stringify(payload),
    });
};

// Update an existing product by ID — Partial<Product> means only the
// changed fields need to be sent, not the full object
export const updateProduct = async (id: number, product: Partial<Product>) => {

      // Convert price from number -> string for backend
    const payload = {
      ...product,
      ...(product.price !== undefined && {
        price: product.price.toString(),
      }),
    };

    return request<Product>(`/update_product/${id}`, {
      method: "PATCH",
      body: JSON.stringify(payload),
    });
};

// Delete a product by ID — returns void since there is no response body
// Caught in inventory/page.tsx handleDelete and shown as a page-level error
export const deleteProduct = async (id: number) => {
    return request<void>(`/delete_product/${id}`, {
      method: "DELETE",
    });
};

//===============================================================================

// ─── Auth ─────────────────────────────────────────────────────────────────────
// Login and register do NOT use request<T> because they don't need the token —
// these are the endpoints that produce the token in the first place.

// POST /Login — sends username and password, returns token and role on success
export const loginUser = async (username: string, password: string): Promise<AuthResponse> => {
  const res = await fetch(`${AUTH_BASE}/login`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ username, password }),
  });
  const data = await res.json().catch(() => ({}));
  // Throw with the backend message so the login page can display it
  if (!res.ok) throw new Error(data.message ?? "Login failed.");
  return data;
};

type RegisterData = {
  username: string;
  password: string;
  fullname: string;
  companyname: string;
  branchname: string;
};

// POST /Register — sends username and password, returns the created username on success
export const registerUser = async (user_data: RegisterData): Promise<AuthResponse> => {
  const res = await fetch(`${AUTH_BASE}/Register`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify( user_data ),
  });
  const data = await res.json().catch(() => ({}));
  // Throw with the backend message so the register page can display it
  if (!res.ok) throw new Error(data.message ?? "Registration failed.");
  return data;
};

// ─── Orders ───────────────────────────────────────────────────────────────────
// Functions for submitting and reading POS transactions.

// Submit a completed sale from the POS checkout.
// The backend is also responsible for deducting stock on each product sold.
export const createOrder = async (order: Omit<SoldItemInfo, "id" | "createdAt">): Promise<SoldItemInfo> => {
  const token = localStorage.getItem("token");

  // Build payload with only the fields Rust expects — no spread to avoid duplicates
  const payload = {
    items: order.items.map((item) => ({
      product_id: item.product_id,       // ← exact name Rust expects
      quantity: item.quantity,
      price: item.price.toString(),    // ← string for Rust
    })),
    total: order.total.toString(),          // ← string for Rust
    amount_paid: order.amount_paid.toString(), // ← string for Rust
    change: order.change.toString(),         // ← string for Rust
  };

  console.log("createOrder payload:", JSON.stringify(payload));

  const res = await fetch(`${POS}/create_order`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      ...(token ? { Authorization: `Bearer ${token}` } : {}),
    },
    body: JSON.stringify(payload),
  });

  const data = await res.json().catch(() => ({}));
  if (!res.ok) throw new Error(data.message ?? `Request failed: ${res.status}`);
  return data;
};
// Fetch all sold items — for an order history page
//export const fetchSoldItems = () =>
//request<SoldItemInfo[]>("/sold_items");


// ─── Dashboard ────────────────────────────────────────────────────────────────
// Fetch all analytics data in a single request.
// The backend aggregates revenue, sales counts, top products,
// recent transactions, and low stock alerts before sending the response.
export const fetchDashboard = async (range?: string): Promise<DashboardSummary> => {
  // Read token from localStorage — set after successful login
  const token = localStorage.getItem("token");

  const url = new URL(`${DASHBOARD}/dashboard`, window.location.origin);

  if (range) {
    url.searchParams.append("range", range);
  }

  const res = await fetch(url.toString(), {
    headers: {
      "Content-Type": "application/json",
      // Attach token for backend authentication
      ...(token ? { Authorization: `Bearer ${token}` } : {}),
    },
  });

  const data = await res.json().catch(() => ({}));

  if (!res.ok) {
    throw new Error(data.message ?? `Request failed: ${res.status}`);
  }

  return data.data;
};