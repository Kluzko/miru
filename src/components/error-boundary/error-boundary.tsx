import React from 'react';
import { Button } from '@/components/ui/button';
import { AlertCircle, RefreshCw, Home } from 'lucide-react';
import { useNavigate } from 'react-router-dom';

interface ErrorInfo {
  componentStack: string;
  errorBoundary?: string;
}

interface ErrorBoundaryState {
  hasError: boolean;
  error: Error | null;
  errorInfo: ErrorInfo | null;
  errorId: string;
}

interface ErrorBoundaryProps {
  children: React.ReactNode;
  fallback?: React.ComponentType<ErrorFallbackProps>;
  onError?: (error: Error, errorInfo: ErrorInfo, errorId: string) => void;
  resetOnPropsChange?: boolean;
  resetKeys?: Array<string | number>;
}

export interface ErrorFallbackProps {
  error: Error;
  errorInfo: ErrorInfo;
  resetError: () => void;
  errorId: string;
}

export class ErrorBoundary extends React.Component<ErrorBoundaryProps, ErrorBoundaryState> {
  private resetTimeoutId: number | null = null;

  constructor(props: ErrorBoundaryProps) {
    super(props);

    this.state = {
      hasError: false,
      error: null,
      errorInfo: null,
      errorId: '',
    };
  }

  static getDerivedStateFromError(error: Error): Partial<ErrorBoundaryState> {
    const errorId = `error_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;

    return {
      hasError: true,
      error,
      errorId,
    };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    const errorId = this.state.errorId;

    this.setState({
      errorInfo,
    });

    // Log to console in development
    if (process.env.NODE_ENV === 'development') {
      console.group(`🚨 Error Boundary Caught Error [${errorId}]`);
      console.error('Error:', error);
      console.error('Component Stack:', errorInfo.componentStack);
      console.groupEnd();
    }

    // Call custom error handler
    this.props.onError?.(error, errorInfo, errorId);
  }

  componentDidUpdate(prevProps: ErrorBoundaryProps) {
    const { resetKeys, resetOnPropsChange } = this.props;
    const { hasError } = this.state;

    if (hasError && prevProps.resetKeys !== resetKeys) {
      if (resetKeys && resetKeys.some((key, index) => prevProps.resetKeys?.[index] !== key)) {
        this.resetErrorBoundary();
      }
    }

    if (hasError && resetOnPropsChange && prevProps.children !== this.props.children) {
      this.resetErrorBoundary();
    }
  }

  resetErrorBoundary = () => {
    if (this.resetTimeoutId) {
      window.clearTimeout(this.resetTimeoutId);
    }

    this.resetTimeoutId = window.setTimeout(() => {
      this.setState({
        hasError: false,
        error: null,
        errorInfo: null,
        errorId: '',
      });
    }, 0);
  };

  render() {
    if (this.state.hasError) {
      const FallbackComponent = this.props.fallback || DefaultErrorFallback;

      return (
        <FallbackComponent
          error={this.state.error!}
          errorInfo={this.state.errorInfo!}
          resetError={this.resetErrorBoundary}
          errorId={this.state.errorId}
        />
      );
    }

    return this.props.children;
  }
}

// Default error fallback component
export function DefaultErrorFallback({
  error,
  errorInfo,
  resetError,
  errorId
}: ErrorFallbackProps) {
  const navigate = useNavigate();

  const handleGoHome = () => {
    navigate('/');
    resetError();
  };

  const handleReload = () => {
    window.location.reload();
  };

  const copyErrorDetails = () => {
    const errorDetails = {
      error: {
        message: error.message,
        stack: error.stack,
        name: error.name,
      },
      errorInfo,
      errorId,
      timestamp: new Date().toISOString(),
      userAgent: navigator.userAgent,
      url: window.location.href,
    };

    navigator.clipboard.writeText(JSON.stringify(errorDetails, null, 2));
  };

  return (
    <div className="flex min-h-[400px] w-full flex-col items-center justify-center p-8 text-center">
      <div className="mx-auto max-w-md space-y-6">
        <div className="flex justify-center">
          <AlertCircle className="h-16 w-16 text-red-500" />
        </div>

        <div className="space-y-2">
          <h2 className="text-2xl font-bold">Something went wrong</h2>
          <p className="text-muted-foreground">
            We're sorry, but something unexpected happened. The error has been logged.
          </p>
        </div>

        {process.env.NODE_ENV === 'development' && (
          <details className="text-left">
            <summary className="cursor-pointer text-sm font-medium">
              Error Details (Development)
            </summary>
            <div className="mt-2 rounded bg-gray-100 p-3 text-xs font-mono dark:bg-gray-800">
              <div className="mb-2">
                <strong>Error ID:</strong> {errorId}
              </div>
              <div className="mb-2">
                <strong>Message:</strong> {error.message}
              </div>
              <div className="mb-2">
                <strong>Stack:</strong>
                <pre className="mt-1 whitespace-pre-wrap">{error.stack}</pre>
              </div>
              {errorInfo && (
                <div>
                  <strong>Component Stack:</strong>
                  <pre className="mt-1 whitespace-pre-wrap">{errorInfo.componentStack}</pre>
                </div>
              )}
            </div>
          </details>
        )}

        <div className="flex flex-col gap-3 sm:flex-row">
          <Button onClick={resetError} variant="default" className="flex items-center gap-2">
            <RefreshCw className="h-4 w-4" />
            Try Again
          </Button>

          <Button onClick={handleGoHome} variant="outline" className="flex items-center gap-2">
            <Home className="h-4 w-4" />
            Go Home
          </Button>

          {process.env.NODE_ENV === 'development' && (
            <Button onClick={copyErrorDetails} variant="outline" size="sm">
              Copy Error Details
            </Button>
          )}

          <Button onClick={handleReload} variant="ghost" size="sm">
            Reload Page
          </Button>
        </div>
      </div>
    </div>
  );
}

// Specialized error boundaries for different contexts
export function AsyncBoundary({ children, ...props }: Omit<ErrorBoundaryProps, 'fallback'>) {
  return (
    <ErrorBoundary fallback={AsyncErrorFallback} {...props}>
      {children}
    </ErrorBoundary>
  );
}

function AsyncErrorFallback({ error, resetError }: ErrorFallbackProps) {
  return (
    <div className="flex min-h-[200px] w-full flex-col items-center justify-center p-6 text-center">
      <AlertCircle className="mb-4 h-12 w-12 text-red-500" />
      <h3 className="mb-2 text-lg font-semibold">Failed to load data</h3>
      <p className="mb-4 text-sm text-muted-foreground">
        {error.message || 'Something went wrong while loading the data.'}
      </p>
      <Button onClick={resetError} size="sm" className="flex items-center gap-2">
        <RefreshCw className="h-4 w-4" />
        Retry
      </Button>
    </div>
  );
}

export function FormBoundary({ children, ...props }: Omit<ErrorBoundaryProps, 'fallback'>) {
  return (
    <ErrorBoundary fallback={FormErrorFallback} {...props}>
      {children}
    </ErrorBoundary>
  );
}

function FormErrorFallback({ error, resetError }: ErrorFallbackProps) {
  return (
    <div className="rounded-lg border border-red-200 bg-red-50 p-4 dark:border-red-800 dark:bg-red-950">
      <div className="flex items-center gap-2">
        <AlertCircle className="h-5 w-5 text-red-600 dark:text-red-400" />
        <h4 className="text-sm font-semibold text-red-800 dark:text-red-200">
          Form Error
        </h4>
      </div>
      <p className="mt-2 text-sm text-red-700 dark:text-red-300">
        {error.message || 'An error occurred while processing the form.'}
      </p>
      <Button
        onClick={resetError}
        size="sm"
        variant="outline"
        className="mt-3 border-red-300 text-red-700 hover:bg-red-100 dark:border-red-700 dark:text-red-300 dark:hover:bg-red-900"
      >
        Try Again
      </Button>
    </div>
  );
}

// Hook for error reporting
export function useErrorHandler() {
  const handleError = React.useCallback((error: Error, errorInfo?: ErrorInfo) => {
    const errorId = `error_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;

    // Log to console
    console.group(`🚨 Manual Error Report [${errorId}]`);
    console.error('Error:', error);
    if (errorInfo) {
      console.error('Error Info:', errorInfo);
    }
    console.groupEnd();

    // Here you could send to your error reporting service
    // Example: Sentry.captureException(error, { extra: errorInfo, tags: { errorId } });

    return errorId;
  }, []);

  return { reportError: handleError };
}
