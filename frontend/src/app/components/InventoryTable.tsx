"use client";
import { Product } from "../lib/api";

interface Props {
  products: Product[];
  onEdit: (product: Product) => void;
  onDelete: (id?: number) => void;
}

export default function InventoryTable({ products, onEdit, onDelete }: Props) {
  if (products.length === 0) {
    return (
      <div className="text-center py-12 text-gray-400">
        <p className="text-lg">No products found</p>
        <p className="text-sm mt-1">Add a product above to get started</p>
      </div>
    );
  }

  return (
    <div className="overflow-x-auto">
      <table className="w-full text-sm">
        <thead>
          <tr className="border-b border-gray-200">
            <th className="text-left py-3 px-4 font-semibold text-gray-600">Name</th>
            <th className="text-left py-3 px-4 font-semibold text-gray-600">SKU</th>
            <th className="text-left py-3 px-4 font-semibold text-gray-600">Category</th>
            <th className="text-right py-3 px-4 font-semibold text-gray-600">Price</th>
            <th className="text-right py-3 px-4 font-semibold text-gray-600">Stock</th>
            <th className="text-right py-3 px-4 font-semibold text-gray-600">Actions</th>
          </tr>
        </thead>
        <tbody>
          {products.map((product) => (
            <tr
              key={product.id}
              className="border-b border-gray-100 hover:bg-gray-50 transition-colors"
            >
              <td className="py-3 px-4 font-medium text-gray-900">{product.name}</td>
              <td className="py-3 px-4 text-gray-500">{product.sku ?? "—"}</td>
              <td className="py-3 px-4 text-gray-500">{product.category ?? "—"}</td>
              <td className="py-3 px-4 text-right text-gray-900">
                ₱{product.price.toFixed(2)}
              </td>
              <td className="py-3 px-4 text-right">
                <span
                  className={`inline-block px-2 py-0.5 rounded-full text-xs font-medium ${
                    product.stock <= 5//ternary operator. If stock is 5 or less, show red badge. If stock is between 6 and 20, show amber badge. Otherwise, show green badge.
                      ? "bg-red-50 text-red-600"
                      : product.stock <= 20
                      ? "bg-amber-50 text-amber-600"
                      : "bg-green-50 text-green-600"
                  }`}
                >
                  {product.stock}
                </span>
              </td>
              <td className="py-3 px-4 text-right">
                <div className="flex justify-end gap-2">
                  <button
                    onClick={() => onEdit(product)}
                    className="px-3 py-1 text-xs font-medium text-blue-600 bg-blue-50 hover:bg-blue-100 rounded-lg transition-colors"
                  >
                    Edit
                  </button>
                  <button
                    onClick={() => onDelete(product.id)}
                    className="px-3 py-1 text-xs font-medium text-red-600 bg-red-50 hover:bg-red-100 rounded-lg transition-colors"
                  >
                    Delete
                  </button>
                </div>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}