// Mark this as a client component so React hooks and browser events work
"use client";
import { useEffect, useState } from "react";
import Sidebar from "../components/Sidebar";
import ProductForm from "../components/ProductForm";
import InventoryTable from "../components/InventoryTable";
import { Product, getProducts, addProduct, updateProduct, deleteProduct } from "../lib/api";

export default function InventoryPage() {
  // Full list of products fetched from the backend
  const [products, setProducts] = useState<Product[]>([]);
  // The product currently being edited — null means no edit is in progress
  const [editing, setEditing] = useState<Product | null>(null);
  // Controls whether the add/edit form panel is visible
  const [showForm, setShowForm] = useState(false);
  // True while the initial product list is being fetched
  const [loading, setLoading] = useState(true);
  // Current value of the search input for filtering the table
  const [search, setSearch] = useState("");
  // Page-level error for fetch and delete failures — shown above the table
  const [error, setError] = useState("");

  // Fetch all products from the API and update state.
  // Called on first load and after every add, update, or delete
  // to keep the table in sync with the database.
  const loadProducts = async () => {
    setError(""); // Clear any previous error before retrying
    try {
      const data = await getProducts();
      setProducts(data);
    } catch (err) {
      // Show the message thrown by getProducts in api.ts
      setError(err instanceof Error ? err.message : "Failed to load products.");
    } finally {
      // Always turn off loading even if the request fails
      setLoading(false);
    }
  };

  // Load products once when the page first mounts
  useEffect(() => {
    loadProducts();
  }, []);

  // Handle submitting the Add Product form.
  // Calls the API to create the product, hides the form, then refreshes the list.
  const handleAdd = async (product: Product) => {
    await addProduct(product);
    setShowForm(false);
    loadProducts();
  };

  // Handle submitting the Edit Product form.
  // Only proceeds if we know which product is being edited (editing.id exists).
  // Calls the API to update, clears the editing state, hides the form, then refreshes.
  const handleUpdate = async (product: Product) => {
    if (editing?.id) {//Guard — do nothing if we don't know which product to update
      await updateProduct(editing.id, product);
      setEditing(null);
      setShowForm(false);
      loadProducts();
    }
  };

  // Handle clicking the Delete button on a table row.
  // Guards against missing id, asks for browser confirmation before proceeding,
  // then calls the API to delete and refreshes the list.
  const handleDelete = async (id?: number) => {
    if (!id) return; // Guard — do nothing if id is missing
    if (!confirm("Delete this product?")) return; // Abort if user cancels confirmation
    setError(""); // Clear any previous error before attempting delete
    try {
      await deleteProduct(id); // Call the API to delete the product
      loadProducts(); // Refresh the table so the deleted row disappears
    } catch (err) {
      // Show the message thrown by deleteProduct in api.ts
      setError(err instanceof Error ? err.message : "Failed to delete product.");
    }
  };

  // Handle clicking the Edit button on a table row.
  // Stores the selected product as the editing target and shows the form
  // pre-filled with that product's current data.
  const handleEdit = (product: Product) => {
    setEditing(product);
    setShowForm(true);
  };

  // Handle clicking the Cancel button inside the form.
  // Clears the editing target and hides the form without saving anything.
  const handleCancel = () => {
    setEditing(null);
    setShowForm(false);
  };

  // Filter the product list client-side based on the search input.
  // Matches against both the product name and category (case-insensitive).
  // Falls back to an empty string if category is undefined.
  const filtered = products.filter((p) =>
    p.name.toLowerCase().includes(search.toLowerCase()) ||
    (p.category ?? "").toLowerCase().includes(search.toLowerCase()) ||
    p.sku.toLowerCase().includes(search.toLowerCase())

  );

  return (
    <div className="flex min-h-screen bg-gray-50">
      {/* Persistent sidebar shared across all pages */}
      <Sidebar />
      <div className="flex-1 p-8 overflow-auto">

        {/* Page header — shows title, total product count, and Add Product button */}
        <div className="flex items-center justify-between mb-8">
          <div>
            <h1 className="text-3xl font-bold text-gray-900">Inventory</h1>
            {/* Pluralises "product" correctly based on count */}
            <p className="text-gray-500 mt-1">
              {products.length} product{products.length !== 1 ? "s" : ""} total
            </p>
          </div>

          {/* Clicking this clears any existing edit state and opens a blank form */}
          <button
            onClick={() => { setEditing(null); setShowForm(true); }}
            className="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white text-sm font-medium rounded-lg transition-colors"
          >
            + Add Product
          </button>
        </div>

        {/* Page-level error banner — shown when fetch or delete fails.
            Not used for add/update since those errors are handled inside ProductForm. */}
        {error && (
          <p className="mb-4 text-sm text-red-500">{error}</p>
        )}

        {/* Form Panel — only rendered when showForm is true.
            The title and submit handler switch between Add and Edit mode
            depending on whether a product is currently being edited. */}
        {showForm && (
          <div className="bg-white rounded-xl border border-gray-200 p-6 mb-6">
            <h2 className="text-lg font-semibold text-gray-800 mb-4">
              {editing ? "Edit Product" : "Add New Product"}
            </h2>
            {/* key forces the form to fully remount when switching between
                adding a new product and editing an existing one,
                so the input fields reset to the correct initial values. */}
            <ProductForm
              key={editing?.id ?? "new"}
              initial={editing ?? undefined}
              onSubmit={editing ? handleUpdate : handleAdd}
              onCancel={handleCancel}
            />
          </div>
        )}

        {/* Search input — filters the table instantly as the user types.
            Only searches locally against the already-loaded product list. */}
        <div className="mb-4">
          <input
            type="text"
            placeholder="Search by name or category..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="w-full sm:w-80 px-3 py-2 border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>

        {/* Product Table — shows a loading message on first fetch,
            then renders the filtered product list with Edit and Delete actions. */}
        <div className="bg-white rounded-xl border border-gray-200 p-6">
          {loading ? (
            <div className="text-center py-12 text-gray-400">Loading products...</div>
          ) : (
            <InventoryTable
              products={filtered}
              onEdit={handleEdit}
              onDelete={handleDelete}
            />
          )}
        </div>
      </div>
    </div>
  );
}
