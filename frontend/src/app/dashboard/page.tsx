// Mark this as a client component so React hooks work
"use client";
import { useEffect, useState } from "react";
import Sidebar from "../components/Sidebar";
import { fetchDashboard, DashboardSummary } from "../lib/api";

export default function DashboardPage() {
  // Holds the full dashboard data returned from the API
  const [summary, setSummary] = useState<DashboardSummary | null>(null);
  // True while the API call is in flight
  const [loading, setLoading] = useState(true);
  const [range, setRange] = useState("day")

  // Fetch dashboard summary once when the page first mounts
  useEffect(() => {
    const fetchSummary = async () => {
      setLoading(true);
      try {
        const data = await fetchDashboard(range);
        setSummary(data);
      } catch (err) {
        console.error(err);
      } finally {
        // Always turn off the loading state whether the request succeeded or failed
        setLoading(false);
      }
    };
    fetchSummary();
  }, [range]);

  // Build the 4 metric cards only when summary data is available.
  // Each card has a label, formatted value, subtitle, and color classes.
  const metrics = summary
    ? [
        {
          label: "Total Revenue",
          // Format as Philippine Peso with 2 decimal places
          value: `₱${summary.total_revenue.toLocaleString("en-PH", { minimumFractionDigits: 2 })}`,
          sub: range,
          color: "text-blue-600",
          bg: "bg-blue-50",
        },
        {
          label: "Quantity Sold",
          value: summary.total_quantity_sold.toLocaleString(),
          sub: range,
          color: "text-green-600",
          bg: "bg-green-50",
        },
        {
          label: "Total Sold",
          value: summary.total_sold.toLocaleString(),
          sub: range,
          color: "text-purple-600",
          bg: "bg-purple-50",
        },
        {
          label: "Avg. Sold Value",
          // Average total per transaction formatted as Peso
          value: `₱${summary.avg_order_value.toLocaleString("en-PH", { minimumFractionDigits: 2 })}`,
          sub: range,
          color: "text-amber-600",
          bg: "bg-amber-50",
        },
      ]
    : [];

  return (
    <div className="flex min-h-screen bg-gray-50">
      {/* Persistent sidebar shared across all pages */}
      <Sidebar />
      <div className="flex-1 p-8 overflow-auto">
        <div className="mb-8">
          <h1 className="text-3xl font-bold text-gray-900">Dashboard1</h1>
          <p className="text-gray-500 mt-1">Business analytics and financial overview</p>
        </div>

        <div>
            <select 
              className="border border-gray-300 rounded-lg px-3 py-2 mb-8"
              value={range}
              onChange={(e) => setRange(e.target.value)}>
              <option value="all" >All</option>
              <option value="day" >Day</option>
              <option value="week" >Week</option>
              <option value="month" >Month</option>
            </select>
            <p className="mt-3"> Selected: {range}</p>
        </div>

        {/* Show a centered loading message while data is being fetched */}
        {loading ? (
          <div className="flex items-center justify-center h-64 text-gray-400">
            Loading...
          </div>
        ) : (
          <>
            {/* ── Metric Cards ───────────────────────────────────────────
                4 cards in a responsive grid:
                - 1 column on mobile
                - 2 columns on small screens
                - 4 columns on large screens
            ─────────────────────────────────────────────────────────── */}
            <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 mb-8">
              {metrics.map((m) => (
                <div
                  key={m.label}
                  className="bg-white rounded-xl border border-gray-200 p-5"
                >
                  {/* Colored badge label at the top of each card */}
                  <div className={`inline-block px-2 py-1 rounded-lg text-xs font-medium mb-3 ${m.bg} ${m.color}`}>
                    {m.label}
                  </div>
                  {/* Main metric value in large bold text */}
                  <p className={`text-2xl font-bold ${m.color}`}>{m.value}</p>
                  {/* Muted subtitle below the value */}
                  <p className="text-xs text-gray-400 mt-1">{m.sub}</p>
                </div>
              ))}
            </div>

            {/* ── Middle Row: Top Products + Low Stock ────────────────── */}
            <div className="grid grid-cols-1 lg:grid-cols-3 gap-6 mb-6">

              {/* Top Selling Products table — takes up 2/3 of the row */}
              <div className="lg:col-span-2 bg-white rounded-xl border border-gray-200 p-6">
                <h2 className="text-lg font-semibold text-gray-800 mb-4">Top Selling Products</h2>
                {/* Show an empty state message if there are no sales yet */}
                {summary?.top_products.length === 0 ? (
                  <p className="text-gray-400 text-sm">No sales data yet.</p>
                ) : (
                  <table className="w-full text-sm">
                    <thead>
                      <tr className="border-b border-gray-100">
                        <th className="text-left py-2 px-3 text-gray-500 font-medium">Product</th>
                        <th className="text-right py-2 px-3 text-gray-500 font-medium">Qty Sold</th>
                        <th className="text-right py-2 px-3 text-gray-500 font-medium">Revenue</th>
                      </tr>
                    </thead>
                    <tbody>
                      {summary?.top_products.map((p, i) => (
                        <tr key={i} className="border-b border-gray-50 hover:bg-gray-50">
                          <td className="py-2 px-3 font-medium text-gray-900">{p.name}</td>
                          <td className="py-2 px-3 text-right text-gray-600">{p.quantity}</td>
                          {/* Revenue formatted as Peso with 2 decimal places */}
                          <td className="py-2 px-3 text-right text-green-600 font-medium">
                            ₱{p.revenue.toLocaleString("en-PH", { minimumFractionDigits: 2 })}
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                )}
              </div>

              {/* Low Stock Alerts — takes up 1/3 of the row.
                  Backend returns products with stock at or below 10. */}
              <div className="bg-white rounded-xl border border-gray-200 p-6">
                <h2 className="text-lg font-semibold text-gray-800 mb-4">Low Stock Alerts</h2>
                {/* Show a positive message if all stock levels are healthy */}
                {summary?.low_stock.length === 0 ? (
                  <p className="text-gray-400 text-sm">All products are well stocked.</p>
                ) : (
                  <div className="flex flex-col gap-2">
                    {summary?.low_stock.map((item, i) => (
                      // Each low-stock product shown as a red-tinted row
                      <div
                        key={i}
                        className="flex items-center justify-between px-3 py-2 bg-red-50 rounded-lg"
                      >
                        <span className="text-sm font-medium text-gray-800 truncate">{item.name}</span>
                        {/* Remaining stock count highlighted in red */}
                        <span className="text-xs font-semibold text-red-600 ml-2 shrink-0">
                          {item.stock} left
                        </span>
                      </div>
                    ))}
                  </div>
                )}
              </div>
            </div>

            {/* ── Recent Transactions ─────────────────────────────────────
                Shows the last 10 orders from the backend, newest first.
            ─────────────────────────────────────────────────────────── */}
            <div className="bg-white rounded-xl border border-gray-200 p-6">
              <h2 className="text-lg font-semibold text-gray-800 mb-4">Recent Transactions</h2>
              {/* Empty state when no orders exist yet */}
              {summary?.recent_transactions.length === 0 ? (
                <p className="text-gray-400 text-sm">No transactions yet.</p>
              ) : (
                <table className="w-full text-sm">
                  <thead>
                    <tr className="border-b border-gray-100">
                      <th className="text-left py-2 px-3 text-gray-500 font-medium">Order #</th>
                      <th className="text-left py-2 px-3 text-gray-500 font-medium">Date</th>
                      <th className="text-right py-2 px-3 text-gray-500 font-medium">Items</th>
                      <th className="text-right py-2 px-3 text-gray-500 font-medium">Total</th>
                    </tr>
                  </thead>
                  <tbody>
                    {summary?.recent_transactions.map((t) => (
                      <tr key={t.id} className="border-b border-gray-50 hover:bg-gray-50">
                        {/* Order ID prefixed with # for readability */}
                        <td className="py-2 px-3 text-gray-900 font-medium">#{t.id}</td>
                        {/* Format ISO date string to a readable Philippine locale date */}
                        <td className="py-2 px-3 text-gray-500">
                          {new Date(t.created_at).toLocaleDateString("en-PH")}
                        </td>
                        {/* Number of distinct items in the order */}
                        <td className="py-2 px-3 text-right text-gray-600">{t.items}</td>
                        {/* Order total formatted as Peso */}
                        <td className="py-2 px-3 text-right font-medium text-gray-900">
                          ₱{t.total.toLocaleString("en-PH", { minimumFractionDigits: 2 })}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              )}
            </div>
          </>
        )}
      </div>
    </div>
  );
}