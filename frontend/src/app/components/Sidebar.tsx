// Mark this as a client component because we use usePathname
// which reads the current URL — a browser-only operation
"use client";
import Link from "next/link";
import { usePathname, useRouter } from "next/navigation";

// Define the 3 navigation items for the sidebar.
// Each item has a display label, the route it links to, and a unicode icon.
// To add or remove pages from the sidebar, just update this array.
const navItems = [
  { label: "Dashboard", href: "/dashboard", icon: "▦" },
  { label: "Inventory", href: "/inventory", icon: "◫" },
  { label: "POS", href: "/pos", icon: "◱" },
];

export default function Sidebar() {
  // Get the current URL path so we can highlight the active nav item.
  // e.g. if the user is on /inventory, pathname === "/inventory"
  const pathname = usePathname();
  const router = useRouter();

  const handleLogout = () => {
    localStorage.removeItem("token"); // clear auth
    router.replace("/"); // prevent back navigation
  }

  return (
    // Full-height sidebar fixed to the left side of the screen.
    // Uses flex-col so the nav stretches and the footer sticks to the bottom.
    <aside className="w-64 min-h-screen bg-white border-r border-gray-200 flex flex-col">

      {/* Brand / logo section at the top of the sidebar */}
      <div className="px-6 py-6 border-b border-gray-200">
        <h1 className="text-xl font-bold text-gray-900">Inventra</h1>
        <p className="text-xs text-gray-400 mt-0.5">Inventory Management</p>
      </div>

      {/* Navigation links — flex-1 makes this section grow to fill
          available space, pushing the footer to the bottom */}
      <nav className="flex-1 px-3 py-4 flex flex-col gap-1">
        {navItems.map((item) => {
          // Check if this nav item matches the current page
          const isActive = pathname === item.href;
          return (
            // Next.js Link handles client-side navigation without a full page reload.
            // The active item gets a blue highlight; inactive items get a gray hover state.
            <Link
              key={item.href}
              href={item.href}
              className={`flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm font-medium transition-colors ${
                isActive
                  ? "bg-blue-50 text-blue-600"       // Active page styling
                  : "text-gray-600 hover:bg-gray-100 hover:text-gray-900" // Inactive styling
              }`}
            >
              {/* Unicode icon displayed to the left of the label */}
              <span className="text-base">{item.icon}</span>
              {item.label}
            </Link>
          );
        })}
      </nav>

      {/* Footer pinned to the bottom of the sidebar showing the logout button and  app version */}
      <div className="px-3 py-4 border-t border-gray-200">
        <button
          onClick={handleLogout}
          className="w-full flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm font-medium text-red-600 hover:bg-red-50 transition-colors"
        >
          <span className="text-base">⎋</span>
          Logout
        </button>

        <p className="text-xs text-gray-400 mt-3">v1.0.0</p>
      </div>
    </aside>
  );
}