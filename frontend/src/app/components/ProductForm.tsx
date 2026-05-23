// Mark this as a client component so React hooks and form events work
"use client";
import { useState, useEffect } from "react";
import { Product } from "../lib/api";


// Props this component accepts:
// - initial: the product to pre-fill the form with when editing (undefined when adding)
// - onSubmit: async function called with the form data when the user saves
// - onCancel: optional function called when the user clicks Cancel
interface Props {
  initial?: Product;
  onSubmit: (product: Product) => Promise<void>;
  onCancel?: () => void;
}

export default function ProductForm({ initial, onSubmit, onCancel}: Props) {
  // Each field is its own piece of state.
  // When initial is provided (edit mode), fields are pre-filled with its values.
  // When initial is undefined (add mode), fields start empty.
  const [name, setName] = useState(initial?.name ?? "");
  const [sku, setSku] = useState(initial?.sku ?? "");
  const [category, setCategory] = useState(initial?.category ?? "");
  // Price and stock are stored as strings because HTML inputs always return strings.
  // They are converted to numbers only when the form is submitted.
  const [price, setPrice] = useState(initial?.price?.toString() ?? "");
  const [stock, setStock] = useState(initial?.stock?.toString() ?? "");
  // True while the onSubmit async call is in flight — disables the save button
  const [loading, setLoading] = useState(false);
  // Holds an error message to display if the save fails
  const [error, setError] = useState("");

  // Sync form fields whenever the initial prop changes.
  // This handles the case where the parent switches from editing one product
  // to editing a different one — the form re-populates with the new product's data.
  useEffect(() => {
    setName(initial?.name ?? "");
    setSku(initial?.sku ?? "");
    setCategory(initial?.category ?? "");
    setPrice(initial?.price?.toString() ?? "");
    setStock(initial?.stock?.toString() ?? "");
  }, [initial]);

  const handleSubmit = async (e: React.FormEvent<HTMLFormElement>) => {
    // Prevent the browser from doing a full page reload on form submit
    e.preventDefault();
    setLoading(true);
    setError("");
    try {
      // Spread initial first so fields like id are preserved when editing,
      // then override with the current form values.
      // Price is parsed to a float and stock to an integer before sending.
      await onSubmit({
        ...initial,
        name,
        sku,
        category,
        price: parseFloat(price),
        stock: parseInt(stock),
      });
      // Only reset the fields after a successful add (not edit).
      // In edit mode we leave the form as-is since the parent will close it.
      // In add mode, we clear the form to make it easy to add another product right away.
      if (!initial) {
        setName("");
        setSku("");
        setCategory("");
        setPrice("");
        setStock("");
      }
    } catch (err) {
      // Show a user-friendly error message if the API call fails
      console.error(err);
      setError(err instanceof Error ? err.message : "Failed to save product.");
    } finally {
      // Always re-enable the button whether the request succeeded or failed
      setLoading(false);
    }
  };

  return (
    // Responsive grid layout:
    // 1 column on mobile, 2 on small screens, 4 on large screens
    <form onSubmit={handleSubmit} className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">

      {/* Error message spans the full width so it's always visible */}
      {error && (
        <p className="col-span-full text-sm text-red-500">{error}</p>
      )}

      {/* Product name — required field */}
      <input
        type="text"
        placeholder="Product name"
        value={name}
        onChange={(e) => setName(e.target.value)}
        className="px-3 py-2 border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
        required
      />

      {/* Product sku — required field */}
      <input
        type="text"
        placeholder="SKU"
        value={sku}
        onChange={(e) => setSku(e.target.value)}
        className="px-3 py-2 border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
        required
      />

      {/* Category — optional, so no required attribute */}
      <input
        type="text"
        placeholder="Category"
        value={category}
        onChange={(e) => setCategory(e.target.value)}
        className="px-3 py-2 border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
      />

      {/* Price — number input with 2 decimal step to allow cents, minimum 0 */}
      <input
        type="number"
        placeholder="Price"
        value={price}
        onChange={(e) => setPrice(e.target.value)}
        className="px-3 py-2 border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
        min="0"
        step="0.01"
        required
      />

      {/* Stock — whole numbers only, minimum 0 */}
      <input
        type="number"
        placeholder="Stock"
        value={stock}
        onChange={(e) => setStock(e.target.value)}
        className="px-3 py-2 border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
        min="0"
        required
      />


      {/* Action buttons — span the full grid width */}
      <div className="col-span-full flex gap-3">
        {/* Submit button — label changes based on mode and loading state:
            "Saving..." while in flight, "Update Product" in edit mode,
            "Add Product" in add mode */}
        <button
          type="submit"
          disabled={loading}
          className="px-5 py-2 bg-blue-600 hover:bg-blue-700 text-white text-sm font-medium rounded-lg transition-colors disabled:opacity-50"
        >
          {loading ? "Saving..." : initial ? "Update Product" : "Add Product"}
        </button>

        {/* Cancel button — only rendered if the parent passed an onCancel handler.
            type="button" prevents it from accidentally triggering form submission. */}
        {onCancel && (
          <button
            type="button"
            onClick={onCancel}
            className="px-5 py-2 bg-gray-100 hover:bg-gray-200 text-gray-700 text-sm font-medium rounded-lg transition-colors"
          >
            Cancel
          </button>
        )}
      </div>
    </form>
  );
}
