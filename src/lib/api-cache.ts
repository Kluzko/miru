import React from "react";
import { QueryClient } from "@tanstack/react-query";

// API Response Caching Layer
// Works alongside React Query for additional caching capabilities

interface CacheEntry<T> {
  data: T;
  timestamp: number;
  ttl: number;
  hits: number;
  lastAccess: number;
}

interface CacheStats {
  hits: number;
  misses: number;
  evictions: number;
  size: number;
}

class APICache {
  private cache = new Map<string, CacheEntry<any>>();
  private stats: CacheStats = { hits: 0, misses: 0, evictions: 0, size: 0 };
  private maxSize: number;
  private defaultTTL: number;

  constructor(maxSize = 100, defaultTTL = 5 * 60 * 1000) {
    // 5 minutes default
    this.maxSize = maxSize;
    this.defaultTTL = defaultTTL;

    // Clean up expired entries every 60 seconds
    setInterval(() => this.cleanup(), 60 * 1000);
  }

  private generateKey(url: string, params?: Record<string, any>): string {
    const paramString = params
      ? JSON.stringify(params, Object.keys(params).sort())
      : "";
    return `${url}:${paramString}`;
  }

  private isExpired(entry: CacheEntry<any>): boolean {
    return Date.now() - entry.timestamp > entry.ttl;
  }

  private evictLRU(): void {
    if (this.cache.size === 0) return;

    let lruKey: string | null = null;
    let oldestAccess = Infinity;

    for (const [key, entry] of this.cache.entries()) {
      if (entry.lastAccess < oldestAccess) {
        oldestAccess = entry.lastAccess;
        lruKey = key;
      }
    }

    if (lruKey) {
      this.cache.delete(lruKey);
      this.stats.evictions++;
      this.stats.size--;
    }
  }

  get<T>(url: string, params?: Record<string, any>): T | null {
    const key = this.generateKey(url, params);
    const entry = this.cache.get(key);

    if (!entry) {
      this.stats.misses++;
      return null;
    }

    if (this.isExpired(entry)) {
      this.cache.delete(key);
      this.stats.misses++;
      this.stats.size--;
      return null;
    }

    // Update access info
    entry.hits++;
    entry.lastAccess = Date.now();
    this.stats.hits++;

    return entry.data;
  }

  set<T>(
    url: string,
    data: T,
    params?: Record<string, any>,
    ttl?: number,
  ): void {
    const key = this.generateKey(url, params);
    const now = Date.now();

    // Evict if at capacity
    if (this.cache.size >= this.maxSize && !this.cache.has(key)) {
      this.evictLRU();
    }

    const isNewEntry = !this.cache.has(key);

    this.cache.set(key, {
      data,
      timestamp: now,
      ttl: ttl ?? this.defaultTTL,
      hits: 0,
      lastAccess: now,
    });

    if (isNewEntry) {
      this.stats.size++;
    }
  }

  invalidate(url: string, params?: Record<string, any>): boolean {
    const key = this.generateKey(url, params);
    const existed = this.cache.has(key);

    if (existed) {
      this.cache.delete(key);
      this.stats.size--;
    }

    return existed;
  }

  invalidateByPattern(pattern: RegExp): number {
    let removed = 0;

    for (const key of this.cache.keys()) {
      if (pattern.test(key)) {
        this.cache.delete(key);
        this.stats.size--;
        removed++;
      }
    }

    return removed;
  }

  clear(): void {
    this.cache.clear();
    this.stats = { hits: 0, misses: 0, evictions: 0, size: 0 };
  }

  private cleanup(): void {
    const now = Date.now();
    let removed = 0;

    for (const [key, entry] of this.cache.entries()) {
      if (now - entry.timestamp > entry.ttl) {
        this.cache.delete(key);
        removed++;
      }
    }

    if (removed > 0) {
      this.stats.size -= removed;
      this.stats.evictions += removed;
    }
  }

  getStats(): CacheStats & { hitRate: number } {
    const totalRequests = this.stats.hits + this.stats.misses;
    return {
      ...this.stats,
      hitRate: totalRequests > 0 ? this.stats.hits / totalRequests : 0,
    };
  }

  // Get popular entries (for debugging/optimization)
  getPopularEntries(
    limit = 10,
  ): Array<{ key: string; hits: number; lastAccess: number }> {
    return Array.from(this.cache.entries())
      .map(([key, entry]) => ({
        key,
        hits: entry.hits,
        lastAccess: entry.lastAccess,
      }))
      .sort((a, b) => b.hits - a.hits)
      .slice(0, limit);
  }
}

// Global cache instance
export const apiCache = new APICache();

// Enhanced fetch wrapper with caching
interface FetchOptions extends RequestInit {
  useCache?: boolean;
  cacheTTL?: number;
  cacheParams?: Record<string, any>;
}

export async function cachedFetch<T>(
  url: string,
  options: FetchOptions = {},
): Promise<T> {
  const { useCache = true, cacheTTL, cacheParams, ...fetchOptions } = options;

  // Try cache first
  if (useCache && fetchOptions.method !== "POST") {
    const cached = apiCache.get<T>(url, cacheParams);
    if (cached) {
      return cached;
    }
  }

  try {
    const response = await fetch(url, fetchOptions);

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`);
    }

    const data = (await response.json()) as T;

    // Cache successful responses
    if (useCache && response.status === 200 && fetchOptions.method !== "POST") {
      apiCache.set(url, data, cacheParams, cacheTTL);
    }

    return data;
  } catch (error) {
    // Don't cache errors
    throw error;
  }
}

// React Query integration
export function createCachedQueryClient() {
  return new QueryClient({
    defaultOptions: {
      queries: {
        staleTime: 5 * 60 * 1000, // 5 minutes
        gcTime: 10 * 60 * 1000, // 10 minutes (was cacheTime)
        retry: (failureCount, error: any) => {
          // Don't retry on 4xx errors
          if (error?.status >= 400 && error?.status < 500) {
            return false;
          }
          return failureCount < 2;
        },
        retryDelay: (attemptIndex) => Math.min(1000 * 2 ** attemptIndex, 30000),
        refetchOnWindowFocus: false,
        refetchOnReconnect: "always",
      },
      mutations: {
        retry: false,
      },
    },
  });
}

// Cache warming utilities
export class CacheWarmer {
  private static instance: CacheWarmer;

  static getInstance(): CacheWarmer {
    if (!CacheWarmer.instance) {
      CacheWarmer.instance = new CacheWarmer();
    }
    return CacheWarmer.instance;
  }

  async warmCache(
    routes: Array<{ url: string; params?: Record<string, any> }>,
  ) {
    const promises = routes.map(async ({ url, params }) => {
      try {
        await cachedFetch(url, { cacheParams: params });
      } catch (error) {
        console.warn(`Failed to warm cache for ${url}:`, error);
      }
    });

    await Promise.allSettled(promises);
  }

  async warmPopularContent() {
    // Pre-load popular/common endpoints
    const popularRoutes = [
      { url: "/api/anime/top?page=1" },
      { url: "/api/collections" },
      { url: "/api/anime/seasonal" },
    ];

    await this.warmCache(popularRoutes);
  }
}

// Cache performance monitor
export class CacheMonitor {
  private static instance: CacheMonitor;
  private metricsHistory: Array<{
    timestamp: number;
    stats: CacheStats & { hitRate: number };
  }> = [];

  static getInstance(): CacheMonitor {
    if (!CacheMonitor.instance) {
      CacheMonitor.instance = new CacheMonitor();
    }
    return CacheMonitor.instance;
  }

  startMonitoring(intervalMs = 60000): void {
    setInterval(() => {
      const stats = apiCache.getStats();
      this.metricsHistory.push({
        timestamp: Date.now(),
        stats,
      });

      // Keep only last 100 entries
      if (this.metricsHistory.length > 100) {
        this.metricsHistory = this.metricsHistory.slice(-100);
      }

      // Log performance in development
      if (process.env.NODE_ENV === "development") {
        console.log("📊 Cache Performance:", {
          hitRate: `${(stats.hitRate * 100).toFixed(1)}%`,
          size: stats.size,
          hits: stats.hits,
          misses: stats.misses,
          evictions: stats.evictions,
        });
      }
    }, intervalMs);
  }

  getMetricsHistory() {
    return [...this.metricsHistory];
  }

  getAverageHitRate(lastNEntries = 10): number {
    const recent = this.metricsHistory.slice(-lastNEntries);
    if (recent.length === 0) return 0;

    const totalHitRate = recent.reduce(
      (sum, entry) => sum + entry.stats.hitRate,
      0,
    );
    return totalHitRate / recent.length;
  }
}

// React hook for cache integration
export function useAPICache() {
  const prefetch = React.useCallback(
    async (url: string, params?: Record<string, any>, ttl?: number) => {
      try {
        await cachedFetch(url, { cacheParams: params, cacheTTL: ttl });
      } catch (error) {
        console.warn("Prefetch failed:", error);
      }
    },
    [],
  );

  const invalidate = React.useCallback(
    (url: string, params?: Record<string, any>) => {
      return apiCache.invalidate(url, params);
    },
    [],
  );

  const invalidatePattern = React.useCallback((pattern: RegExp) => {
    return apiCache.invalidateByPattern(pattern);
  }, []);

  const getStats = React.useCallback(() => {
    return apiCache.getStats();
  }, []);

  return {
    prefetch,
    invalidate,
    invalidatePattern,
    getStats,
  };
}

// Initialize cache monitoring in development
if (process.env.NODE_ENV === "development") {
  CacheMonitor.getInstance().startMonitoring();
}

export default apiCache;
