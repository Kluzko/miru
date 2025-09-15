import React, {
  useMemo,
  useCallback,
  memo,
  lazy,
  Suspense,
  useDeferredValue,
  useTransition,
} from "react";
import { LoadingState } from "@/components/ui/loading";

// Performance optimization utilities

/**
 * Creates a memoized component with custom comparison
 */
export function createMemoComponent<P>(
  component: React.ComponentType<P>,
  propsAreEqual?: (prevProps: P, nextProps: P) => boolean,
) {
  const MemoizedComponent = memo(component, propsAreEqual);
  MemoizedComponent.displayName = `Memo(${component.displayName || component.name})`;
  return MemoizedComponent;
}

/**
 * Creates a lazy-loaded component with custom loading fallback
 */
export function createLazyComponent<P extends Record<string, any> = {}>(
  importFn: () => Promise<{ default: React.ComponentType<P> }>,
  fallback?: React.ComponentType,
) {
  const LazyComponent = lazy(importFn);
  const FallbackComponent = fallback || LoadingState;

  return memo((props: P) => (
    <Suspense fallback={<FallbackComponent />}>
      <LazyComponent {...props} />
    </Suspense>
  ));
}

/**
 * Hook for creating stable callback references
 */
export function useStableCallback<T extends (...args: any[]) => any>(
  callback: T,
  deps: React.DependencyList,
): T {
  return useCallback(callback, deps);
}

/**
 * Hook for creating stable object references
 */
export function useStableMemo<T>(
  factory: () => T,
  deps: React.DependencyList,
): T {
  return useMemo(factory, deps);
}

/**
 * Hook for debounced values (useful for search inputs)
 */
export function useDebouncedValue<T>(value: T, delay: number): T {
  const [debouncedValue, setDebouncedValue] = React.useState(value);

  React.useEffect(() => {
    const handler = setTimeout(() => {
      setDebouncedValue(value);
    }, delay);

    return () => {
      clearTimeout(handler);
    };
  }, [value, delay]);

  return debouncedValue;
}

/**
 * Hook for transitions with loading state
 */
export function useTransitionWithLoading() {
  const [isPending, startTransition] = useTransition();

  const startTransitionWithLoading = useCallback((callback: () => void) => {
    startTransition(callback);
  }, []);

  return [isPending, startTransitionWithLoading] as const;
}

/**
 * Hook for deferred values (React 18)
 */
export function useDeferredSearch(query: string) {
  const deferredQuery = useDeferredValue(query);
  const isStale = query !== deferredQuery;

  return { deferredQuery, isStale };
}

/**
 * Higher-order component for virtual scrolling optimization
 */
interface VirtualizedListProps<T> {
  items: T[];
  itemHeight: number;
  containerHeight: number;
  renderItem: (item: T, index: number) => React.ReactNode;
  overscan?: number;
  className?: string;
}

export function VirtualizedList<T>({
  items,
  itemHeight,
  containerHeight,
  renderItem,
  overscan = 5,
  className,
}: VirtualizedListProps<T>) {
  const [scrollTop, setScrollTop] = React.useState(0);

  const visibleRange = useMemo(() => {
    const start = Math.floor(scrollTop / itemHeight);
    const end = Math.min(
      start + Math.ceil(containerHeight / itemHeight) + overscan,
      items.length,
    );
    return { start: Math.max(0, start - overscan), end };
  }, [scrollTop, itemHeight, containerHeight, overscan, items.length]);

  const handleScroll = useCallback((e: React.UIEvent<HTMLDivElement>) => {
    setScrollTop(e.currentTarget.scrollTop);
  }, []);

  const visibleItems = useMemo(() => {
    return items
      .slice(visibleRange.start, visibleRange.end)
      .map((item, index) => ({
        item,
        index: visibleRange.start + index,
      }));
  }, [items, visibleRange.start, visibleRange.end]);

  return (
    <div
      className={className}
      style={{ height: containerHeight, overflow: "auto" }}
      onScroll={handleScroll}
    >
      <div style={{ height: items.length * itemHeight, position: "relative" }}>
        {visibleItems.map(({ item, index }) => (
          <div
            key={index}
            style={{
              position: "absolute",
              top: index * itemHeight,
              height: itemHeight,
              width: "100%",
            }}
          >
            {renderItem(item, index)}
          </div>
        ))}
      </div>
    </div>
  );
}

/**
 * Performance monitoring hook (development only)
 */
export function usePerformanceMonitor(componentName: string) {
  const renderStart = React.useRef<number>(0);
  const renderCount = React.useRef<number>(0);

  React.useEffect(() => {
    if (process.env.NODE_ENV === "development") {
      renderCount.current += 1;
      const renderEnd = performance.now();
      const renderTime = renderEnd - renderStart.current;

      console.log(
        `🎯 ${componentName} render #${renderCount.current}: ${renderTime.toFixed(2)}ms`,
      );
    }
  });

  if (process.env.NODE_ENV === "development") {
    renderStart.current = performance.now();
  }
}

/**
 * Image lazy loading component
 */
interface LazyImageProps extends React.ImgHTMLAttributes<HTMLImageElement> {
  src: string;
  fallbackSrc?: string;
  threshold?: number;
  className?: string;
}

export const LazyImage = memo<LazyImageProps>(
  ({ src, fallbackSrc, threshold = 0.1, className, ...props }) => {
    const [isLoaded, setIsLoaded] = React.useState(false);
    const [isInView, setIsInView] = React.useState(false);
    const [hasError, setHasError] = React.useState(false);
    const imgRef = React.useRef<HTMLImageElement>(null);

    React.useEffect(() => {
      const img = imgRef.current;
      if (!img) return;

      const observer = new IntersectionObserver(
        ([entry]) => {
          if (entry.isIntersecting) {
            setIsInView(true);
            observer.disconnect();
          }
        },
        { threshold },
      );

      observer.observe(img);
      return () => observer.disconnect();
    }, [threshold]);

    const handleLoad = useCallback(() => {
      setIsLoaded(true);
    }, []);

    const handleError = useCallback(() => {
      setHasError(true);
    }, []);

    const imageSrc = hasError && fallbackSrc ? fallbackSrc : src;

    return (
      <img
        ref={imgRef}
        src={isInView ? imageSrc : undefined}
        onLoad={handleLoad}
        onError={handleError}
        className={className}
        style={{
          opacity: isLoaded ? 1 : 0,
          transition: "opacity 0.3s ease-in-out",
        }}
        {...props}
      />
    );
  },
);

LazyImage.displayName = "LazyImage";

/**
 * Memoized list component with stable keys
 */
interface MemoizedListProps<T> {
  items: T[];
  renderItem: (item: T, index: number) => React.ReactNode;
  getItemKey: (item: T, index: number) => React.Key;
  className?: string;
}

export function MemoizedList<T>({
  items,
  renderItem,
  getItemKey,
  className,
}: MemoizedListProps<T>) {
  const renderedItems = useMemo(() => {
    return items.map((item, index) => (
      <React.Fragment key={getItemKey(item, index)}>
        {renderItem(item, index)}
      </React.Fragment>
    ));
  }, [items, renderItem, getItemKey]);

  return <div className={className}>{renderedItems}</div>;
}

/**
 * Error boundary with performance monitoring
 */
interface PerformanceErrorBoundaryState {
  hasError: boolean;
  errorDetails?: {
    error: Error;
    renderTime: number;
    componentStack: string;
  };
}

export class PerformanceErrorBoundary extends React.Component<
  { children: React.ReactNode; componentName?: string },
  PerformanceErrorBoundaryState
> {
  private renderStart = 0;

  constructor(props: { children: React.ReactNode; componentName?: string }) {
    super(props);
    this.state = { hasError: false };
    this.renderStart = performance.now();
  }

  static getDerivedStateFromError(_: Error): PerformanceErrorBoundaryState {
    return {
      hasError: true,
    };
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    const renderTime = performance.now() - this.renderStart;

    this.setState({
      errorDetails: {
        error,
        renderTime,
        componentStack: errorInfo.componentStack || "",
      },
    });

    if (process.env.NODE_ENV === "development") {
      console.error(`⚠️ Performance Error in ${this.props.componentName}:`, {
        error: error.message,
        renderTime: `${renderTime.toFixed(2)}ms`,
        stack: error.stack,
      });
    }
  }

  render() {
    if (this.state.hasError) {
      return (
        <div className="rounded-lg border border-red-200 bg-red-50 p-4">
          <h3 className="text-sm font-semibold text-red-800">
            Component Error{" "}
            {this.props.componentName && `in ${this.props.componentName}`}
          </h3>
          {process.env.NODE_ENV === "development" &&
            this.state.errorDetails && (
              <details className="mt-2">
                <summary className="cursor-pointer text-xs">Debug Info</summary>
                <pre className="mt-1 text-xs">
                  Render time: {this.state.errorDetails.renderTime.toFixed(2)}ms
                  {"\n"}
                  Error: {this.state.errorDetails.error.message}
                </pre>
              </details>
            )}
        </div>
      );
    }

    return this.props.children;
  }
}
