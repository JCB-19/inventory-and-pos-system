// Mark this as a client component so React hooks and browser events work
"use client";
import { useEffect, useState } from "react";
import Sidebar from "../components/Sidebar";
import { getProducts, createOrder, Product } from "../lib/api";

// Extend Product with a quantity field to track how many are in the cart
interface CartItem extends Product {
  quantity: number;
}

export default function POSPage() {
  // All available products loaded from the database
  const [products, setProducts] = useState<Product[]>([]);
  // Items currently added to the cart
  const [cart, setCart] = useState<CartItem[]>([]);
  // Search input value for filtering the product grid
  const [search, setSearch] = useState("");
  // True while products are being fetched on first load
  const [loading, setLoading] = useState(true);
  // True while the checkout request is in flight
  const [processing, setProcessing] = useState(false);
  // True briefly after a successful order to show a success message
  const [success, setSuccess] = useState(false);
  // The cash amount the customer hands over
  const [amountPaid, setAmountPaid] = useState("0");

  // Fetch all products once when the page first loads
  useEffect(() => {
    console.log("token:", localStorage.getItem("token"));
    const load = async () => {
      try {
        const data = await getProducts();
        setProducts(data);
      } catch (err) {
        console.error(err);
      } finally {
        setLoading(false);
      }
    };
    load();
  }, []);

  // Add a product to the cart or increment its quantity if already present.
  // Prevents adding more than the available stock.
  const addToCart = (product: Product) => {
    setCart((prev) => {
      const existing = prev.find((i) => i.id === product.id);
      if (existing) {
        // Already in cart — don't exceed stock
        if (existing.quantity >= product.stock) return prev;
        return prev.map((i) =>
          i.id === product.id ? { ...i, quantity: i.quantity + 1 } : i
        );
      }
      // Not in cart yet — add with quantity 1
      return [...prev, { ...product, quantity: 1 }];
    });
  };

  // Update the quantity of a cart item.
  // If qty drops to 0 or below, remove the item from the cart entirely.
  const updateQty = (id: number | undefined, qty: number) => {
    if (!id) return;
    if (qty <= 0) {
      setCart((prev) => prev.filter((i) => i.id !== id));
    } else {
      setCart((prev) =>
        prev.map((i) => (i.id === id ? { ...i, quantity: qty } : i))
      );
    }
  };

  // Remove a specific item from the cart by its product id
  const removeFromCart = (id?: number) => {
    setCart((prev) => prev.filter((i) => i.id !== id));
  };

  // Compute order totals
  const subtotal = cart.reduce((sum, i) => sum + i.price * i.quantity, 0);
  const tax = subtotal * 0.12; // 12% VAT
  const total = subtotal + tax;
  const change = parseFloat(amountPaid || "0") - total; // Change to give back to customer

  // Submit the order to the API, then reset the cart and refresh products
  // so stock levels reflect the sale
  const handleCheckout = async () => {
    if (cart.length === 0) return;
    setProcessing(true);
    try {
      await createOrder({
        items: cart.map((i) => ({
          product_id: i.id!,
          quantity: i.quantity,
          price: i.price,
        })),
        total,
        amount_paid: parseFloat(amountPaid || "0"),
        change,
      });
      // Order succeeded — reset UI state
      setSuccess(true);
      setCart([]);
      setAmountPaid("0");
      // Hide the success message after 3 seconds
      setTimeout(() => setSuccess(false), 3000);
      // Refresh products so updated stock counts are shown
      const data = await getProducts();
      setProducts(data);
    } catch (err) {
      console.error(err);
    } finally {
      setProcessing(false);
    }
  };

  // Filter products for the grid: only show in-stock items that match the search query
  const filtered = products.filter(
    (p) =>
      p.stock > 0 &&
      (p.name.toLowerCase().includes(search.toLowerCase()) ||
      (p.category ?? "").toLowerCase().includes(search.toLowerCase()) ||
      p.sku.toLowerCase().includes(search.toLowerCase()))
  );

  return (
    <div className="flex min-h-screen bg-gray-50">
      <Sidebar />
      <div className="flex-1 flex overflow-hidden">

        {/* Left panel — product grid for selecting items */}
        <div className="flex-1 p-6 overflow-auto">
          <div className="mb-6">
            <h1 className="text-3xl font-bold text-gray-900">Point of Sale</h1>
            <p className="text-gray-500 mt-1">Select products to add to cart</p>
          </div>

          {/* Search input to filter products by name or category */}
          <input
            type="text"
            placeholder="Search products..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="w-full sm:w-80 px-3 py-2 border border-gray-200 rounded-lg text-sm mb-6 focus:outline-none focus:ring-2 focus:ring-blue-500"
          />

          {/* Show a loading state while products are being fetched */}
          {loading ? (
            <div className="text-gray-400 text-center py-12">Loading products...</div>
          ) : (
            <div className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 gap-4">
              {/* Each product card is a button — clicking it adds to cart */}
              {filtered.map((product) => (
                <button
                  key={product.id}
                  onClick={() => addToCart(product)}
                  className="bg-white border border-gray-200 rounded-xl p-4 text-left hover:border-blue-400 hover:shadow-sm transition-all group"
                >
                  {/* Category badge acts as a placeholder image */}
                  <div className="w-full h-10 bg-gray-100 rounded-lg mb-3 flex items-center justify-center text-gray-400 text-xs group-hover:bg-blue-50">
                    {product.category ?? "Item"}
                  </div>
                  <p className="font-semibold text-gray-900 text-sm truncate">{product.name}</p>
                  <p className="text-blue-600 font-bold text-sm mt-1">
                    ₱{product.price.toFixed(2)}
                  </p>
                  <p className="text-xs text-gray-400 mt-0.5">{product.stock} in stock</p>
                </button>
              ))}
              {/* Empty state when no products match the search */}
              {filtered.length === 0 && (
                <p className="col-span-full text-center text-gray-400 py-12">
                  No products available.
                </p>
              )}
            </div>
          )}
        </div>

        {/* Right panel — cart, totals, and checkout */}
        <div className="w-80 bg-white border-l border-gray-200 flex flex-col">
          <div className="px-5 py-4 border-b border-gray-200">
            <h2 className="text-lg font-semibold text-gray-800">Cart</h2>
            {/* Show how many distinct items are in the cart */}
            <p className="text-xs text-gray-400">{cart.length} item{cart.length !== 1 ? "s" : ""}</p>
          </div>

          {/* Scrollable list of cart items */}
          <div className="flex-1 overflow-auto px-4 py-3 flex flex-col gap-3">
            {cart.length === 0 ? (
              <p className="text-center text-gray-400 text-sm mt-8">Cart is empty</p>
            ) : (
              cart.map((item) => (
                <div key={item.id} className="flex items-center gap-3 bg-gray-50 rounded-lg px-3 py-2">
                  <div className="flex-1 min-w-0">
                    <p className="text-sm font-medium text-gray-900 truncate">{item.name}</p>
                    <p className="text-xs text-gray-500">₱{item.price.toFixed(2)} each</p>
                  </div>
                  {/* Quantity stepper — minus, count, plus */}
                  <div className="flex items-center gap-1">
                    <button
                      onClick={() => updateQty(item.id, item.quantity - 1)}
                      className="w-6 h-6 rounded bg-gray-200 hover:bg-gray-300 text-gray-700 text-sm font-bold flex items-center justify-center"
                    >
                      −
                    </button>
                    <span className="w-6 text-center text-sm font-medium">{item.quantity}</span>
                    <button
                      onClick={() => updateQty(item.id, item.quantity + 1)}
                      // Disable the plus button when quantity reaches available stock
                      disabled={item.quantity >= item.stock}
                      className="w-6 h-6 rounded bg-gray-200 hover:bg-gray-300 text-gray-700 text-sm font-bold flex items-center justify-center disabled:opacity-40"
                    >
                      +
                    </button>
                  </div>
                  {/* Remove item from cart entirely */}
                  <button
                    onClick={() => removeFromCart(item.id)}
                    className="text-red-400 hover:text-red-600 text-xs ml-1"
                  >
                    ✕
                  </button>
                </div>
              ))
            )}
          </div>

          {/* Order summary, payment input, and checkout button */}
          <div className="px-5 py-4 border-t border-gray-200 flex flex-col gap-2">
            {/* Subtotal before tax */}
            <div className="flex justify-between text-sm text-gray-600">
              <span>Subtotal</span>
              <span>₱{subtotal.toFixed(2)}</span>
            </div>
            {/* 12% VAT */}
            <div className="flex justify-between text-sm text-gray-600">
              <span>VAT (12%)</span>
              <span>₱{tax.toFixed(2)}</span>
            </div>
            {/* Grand total including tax */}
            <div className="flex justify-between text-base font-bold text-gray-900 border-t border-gray-200 pt-2 mt-1">
              <span>Total</span>
              <span>₱{total.toFixed(2)}</span>
            </div>

            {/* Cashier enters the amount the customer hands over */}
            <input
              type="number"
              placeholder="Amount paid"
              value={amountPaid}
              onChange={(e) => setAmountPaid(e.target.value)}
              className="mt-2 px-3 py-2 border border-gray-200 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 w-full"
              min="0"
              step="0.01"
            />

            {/* Only show change due once the customer has paid enough */}
            {parseFloat(amountPaid || "0") >= total && total > 0 && (
              <div className="flex justify-between text-sm font-semibold text-green-600 bg-green-50 px-3 py-2 rounded-lg">
                <span>Change</span>
                <span>₱{change.toFixed(2)}</span>
              </div>
            )}

            {/* Success banner shown for 3 seconds after a completed order */}
            {success && (
              <div className="text-center text-sm font-medium text-green-600 bg-green-50 px-3 py-2 rounded-lg">
                ✓ Order placed successfully!
              </div>
            )}

            {/* Checkout button — disabled if cart is empty, still processing, or underpaid */}
            <button
              onClick={handleCheckout}
              suppressHydrationWarning
              disabled={
                cart.length === 0 ||
                processing ||
                parseFloat(amountPaid || "0") < total
              }
              className="mt-2 w-full py-3 bg-blue-600 hover:bg-blue-700 disabled:opacity-50 text-white font-semibold rounded-lg transition-colors text-sm"
            >
              {processing ? "Processing..." : "Checkout"}
            </button>

            {/* Clear cart button — only visible when there are items in the cart */}
            {cart.length > 0 && (
              <button
                onClick={() => setCart([])}
                className="w-full py-2 text-sm text-gray-500 hover:text-red-500 transition-colors"
              >
                Clear cart
              </button>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}