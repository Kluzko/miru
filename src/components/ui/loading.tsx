import React from 'react';
import { Loader2 } from 'lucide-react';
import { cn } from '@/lib/utils';

interface LoadingSpinnerProps {
  size?: 'sm' | 'md' | 'lg';
  className?: string;
}

export const LoadingSpinner = React.memo<LoadingSpinnerProps>(({
  size = 'md',
  className
}) => {
  const sizeClasses = {
    sm: 'h-4 w-4',
    md: 'h-6 w-6',
    lg: 'h-8 w-8'
  };

  return (
    <Loader2
      className={cn(
        'animate-spin text-muted-foreground',
        sizeClasses[size],
        className
      )}
    />
  );
});

LoadingSpinner.displayName = 'LoadingSpinner';

interface LoadingStateProps {
  title?: string;
  description?: string;
  size?: 'sm' | 'md' | 'lg';
  className?: string;
}

export const LoadingState = React.memo<LoadingStateProps>(({
  title = 'Loading...',
  description,
  size = 'md',
  className
}) => {
  return (
    <div className={cn(
      'flex flex-col items-center justify-center p-8 text-center',
      className
    )}>
      <LoadingSpinner size={size} className="mb-4" />
      <h3 className="text-lg font-medium">{title}</h3>
      {description && (
        <p className="mt-1 text-sm text-muted-foreground">{description}</p>
      )}
    </div>
  );
});

LoadingState.displayName = 'LoadingState';

interface SkeletonProps {
  className?: string;
  variant?: 'text' | 'circular' | 'rectangular';
  width?: number | string;
  height?: number | string;
  animation?: 'pulse' | 'wave';
}

export const Skeleton = React.memo<SkeletonProps>(({
  className,
  variant = 'text',
  width,
  height,
  animation = 'pulse'
}) => {
  const baseClasses = 'bg-muted';
  const animationClasses = {
    pulse: 'animate-pulse',
    wave: 'animate-pulse' // Could implement wave animation if needed
  };

  const variantClasses = {
    text: 'rounded',
    circular: 'rounded-full',
    rectangular: 'rounded-md'
  };

  const style: React.CSSProperties = {};
  if (width) style.width = width;
  if (height) style.height = height;

  return (
    <div
      className={cn(
        baseClasses,
        animationClasses[animation],
        variantClasses[variant],
        variant === 'text' && !height && 'h-4',
        variant === 'text' && !width && 'w-full',
        className
      )}
      style={style}
    />
  );
});

Skeleton.displayName = 'Skeleton';

// Specialized skeleton components
export const AnimeCardSkeleton = React.memo(() => (
  <div className="space-y-3">
    <Skeleton variant="rectangular" className="aspect-[3/4] w-full" />
    <div className="space-y-2">
      <Skeleton className="h-4 w-3/4" />
      <Skeleton className="h-3 w-1/2" />
    </div>
  </div>
));

AnimeCardSkeleton.displayName = 'AnimeCardSkeleton';

export const AnimeListSkeleton = React.memo<{ count?: number }>(({ count = 6 }) => (
  <div className="grid gap-6 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5">
    {Array.from({ length: count }, (_, i) => (
      <AnimeCardSkeleton key={i} />
    ))}
  </div>
));

AnimeListSkeleton.displayName = 'AnimeListSkeleton';

export const TableRowSkeleton = React.memo<{ columns?: number }>(({ columns = 4 }) => (
  <tr>
    {Array.from({ length: columns }, (_, i) => (
      <td key={i} className="px-4 py-3">
        <Skeleton className="h-4" />
      </td>
    ))}
  </tr>
));

TableRowSkeleton.displayName = 'TableRowSkeleton';

interface InlineLoadingProps {
  text?: string;
  size?: 'sm' | 'md';
  className?: string;
}

export const InlineLoading = React.memo<InlineLoadingProps>(({
  text = 'Loading',
  size = 'sm',
  className
}) => (
  <span className={cn('inline-flex items-center gap-2', className)}>
    <LoadingSpinner size={size} />
    <span className="text-sm text-muted-foreground">{text}</span>
  </span>
));

InlineLoading.displayName = 'InlineLoading';

// Higher-order component for adding loading states
export function withLoading<P extends object>(
  Component: React.ComponentType<P>
) {
  const WithLoadingComponent = React.memo((
    props: P & {
      isLoading?: boolean;
      loadingComponent?: React.ComponentType;
      loadingProps?: any;
    }
  ) => {
    const { isLoading, loadingComponent: LoadingComponent = LoadingState, loadingProps, ...restProps } = props;

    if (isLoading) {
      return <LoadingComponent {...loadingProps} />;
    }

    return <Component {...(restProps as P)} />;
  });

  WithLoadingComponent.displayName = `withLoading(${Component.displayName || Component.name})`;

  return WithLoadingComponent;
}
