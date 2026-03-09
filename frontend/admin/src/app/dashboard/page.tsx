"use client";

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import { adminApi } from "@/lib/api";

interface Stats {
  orders: number;
  products: number;
  customers: number;
}

export default function DashboardPage() {
  const router = useRouter();
  const [stats, setStats] = useState<Stats | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const token =
      typeof window !== "undefined"
        ? localStorage.getItem("medusa_admin_token")
        : null;

    if (!token) {
      router.replace("/login");
      return;
    }

    async function loadStats() {
      try {
        const [ordersRes, productsRes, customersRes] = await Promise.allSettled([
          adminApi.get("/admin/orders?limit=1"),
          adminApi.get("/admin/products?limit=1"),
          adminApi.get("/admin/customers?limit=1"),
        ]);

        setStats({
          orders:
            ordersRes.status === "fulfilled"
              ? (ordersRes.value.data?.count ?? 0)
              : 0,
          products:
            productsRes.status === "fulfilled"
              ? (productsRes.value.data?.count ?? 0)
              : 0,
          customers:
            customersRes.status === "fulfilled"
              ? (customersRes.value.data?.count ?? 0)
              : 0,
        });
      } catch (err: unknown) {
        setError(err instanceof Error ? err.message : "Failed to load stats");
      }
    }

    loadStats();
  }, [router]);

  function handleLogout() {
    localStorage.removeItem("medusa_admin_token");
    // Also clear the auth cookie used by the middleware.
    document.cookie = "medusa_admin_token=; path=/; max-age=0; SameSite=Lax";
    router.push("/login");
  }

  return (
    <div className="min-h-screen bg-gray-50">
      {/* Nav */}
      <nav className="bg-white shadow">
        <div className="mx-auto flex max-w-7xl items-center justify-between px-4 py-3">
          <span className="text-lg font-semibold text-gray-900">
            MedusaRust Admin
          </span>
          <button
            onClick={handleLogout}
            className="rounded-md bg-gray-100 px-3 py-1.5 text-sm font-medium text-gray-700
                       hover:bg-gray-200"
          >
            Sign out
          </button>
        </div>
      </nav>

      {/* Content */}
      <main className="mx-auto max-w-7xl px-4 py-8">
        <h2 className="mb-6 text-xl font-semibold text-gray-900">Dashboard</h2>

        {error && (
          <p className="mb-4 rounded-md bg-red-50 px-4 py-3 text-sm text-red-600">
            {error}
          </p>
        )}

        <div className="grid gap-6 sm:grid-cols-3">
          <StatCard label="Orders" value={stats?.orders} />
          <StatCard label="Products" value={stats?.products} />
          <StatCard label="Customers" value={stats?.customers} />
        </div>

        <p className="mt-8 text-sm text-gray-400">
          Connected to:{" "}
          <code className="font-mono">
            {process.env.NEXT_PUBLIC_API_URL ?? "http://localhost:9000"}
          </code>
        </p>
      </main>
    </div>
  );
}

function StatCard({
  label,
  value,
}: {
  label: string;
  value: number | undefined;
}) {
  return (
    <div className="rounded-xl bg-white p-6 shadow">
      <p className="text-sm font-medium text-gray-500">{label}</p>
      <p className="mt-1 text-3xl font-bold text-gray-900">
        {value === undefined ? "—" : value.toLocaleString()}
      </p>
    </div>
  );
}
