// Re-export all service hooks
export * from './authClient';
export * from './orderClient';
export * from './userClient';

// Export authentication system
export * from './auth-manager';

// Axios configuration for the generated hooks
import axios, { AxiosResponse, InternalAxiosRequestConfig } from 'axios';
import { AuthManager } from './auth-manager';

export interface RabbitMeshClientConfig {
  baseURL: string;
  timeout?: number;
  headers?: Record<string, string>;
  auth?: {
    /** Enable automatic JWT token management */
    autoManageTokens?: boolean;
    /** Enable RBAC (Role-Based Access Control) support */
    rbacEnabled?: boolean;
    /** Enable ABAC (Attribute-Based Access Control) support */
    abacEnabled?: boolean;
    /** Custom token storage (defaults to localStorage) */
    tokenStorage?: 'localStorage' | 'sessionStorage' | 'memory';
    /** Custom refresh token threshold (minutes before expiry to refresh) */
    refreshThreshold?: number;
  };
}

// Global auth manager instance
let authManager: AuthManager;

// Configure axios defaults for all generated hooks with comprehensive auth support
export function configureRabbitMeshClient(config: RabbitMeshClientConfig | string) {
  const clientConfig = typeof config === 'string' 
    ? { baseURL: config, auth: { autoManageTokens: true } }
    : config;

  // Initialize auth manager with configuration
  authManager = new AuthManager({
    autoManageTokens: clientConfig.auth?.autoManageTokens ?? true,
    rbacEnabled: clientConfig.auth?.rbacEnabled ?? true,
    abacEnabled: clientConfig.auth?.abacEnabled ?? true,
    tokenStorage: clientConfig.auth?.tokenStorage ?? 'localStorage',
    refreshThreshold: clientConfig.auth?.refreshThreshold ?? 5,
  });

  // Configure axios defaults
  axios.defaults.baseURL = clientConfig.baseURL;
  axios.defaults.timeout = clientConfig.timeout || 10000;
  axios.defaults.headers.common['Content-Type'] = 'application/json';
  
  if (clientConfig.headers) {
    Object.assign(axios.defaults.headers.common, clientConfig.headers);
  }

  // Setup automatic JWT token injection for all requests
  axios.interceptors.request.use(
    async (config: InternalAxiosRequestConfig): Promise<InternalAxiosRequestConfig> => {
      return authManager.injectAuthHeaders(config);
    },
    (error) => Promise.reject(error)
  );

  // Setup automatic token refresh and auth error handling
  axios.interceptors.response.use(
    (response: AxiosResponse) => {
      // Handle successful login responses automatically
      authManager.handleAuthResponse(response);
      return response;
    },
    async (error) => {
      return authManager.handleAuthError(error);
    }
  );
}

// Export auth manager utilities for manual use if needed
export function getAuthManager(): AuthManager | undefined {
  return authManager;
}

// Convenience functions for common auth operations
export const auth = {
  /** Get current authentication status */
  isAuthenticated: () => authManager?.isAuthenticated() ?? false,
  
  /** Get current user info */
  getCurrentUser: () => authManager?.getCurrentUser() ?? null,
  
  /** Get current JWT token */
  getToken: () => authManager?.getAccessToken() ?? null,
  
  /** Get current user role (RBAC) */
  getRole: () => authManager?.getRole() ?? null,
  
  /** Get current user permissions (RBAC) */
  getPermissions: () => authManager?.getPermissions() ?? [],
  
  /** Check if user has specific permission (RBAC) */
  hasPermission: (permission: string) => authManager?.hasPermission(permission) ?? false,
  
  /** Check if user has specific role (RBAC) */
  hasRole: (role: string) => authManager?.hasRole(role) ?? false,
  
  /** Get user attributes (ABAC) */
  getAttributes: () => authManager?.getAttributes() ?? {},
  
  /** Check ABAC policy */
  checkPolicy: (resource: string, action: string, context?: Record<string, any>) => 
    authManager?.checkAbacPolicy(resource, action, context) ?? false,
  
  /** Manually logout */
  logout: () => authManager?.logout(),
  
  /** Manually refresh token */
  refreshToken: () => authManager?.refreshAccessToken(),
};

// Example usage:
// import { configureRabbitMeshClient, auth, useLogin, useCreateOrder } from '@your-org/rabbitmesh-client';
// 
// // Configure once in your app initialization with full auth support
// configureRabbitMeshClient({
//   baseURL: 'http://localhost:3333',
//   auth: {
//     autoManageTokens: true,    // Automatic JWT management
//     rbacEnabled: true,         // Role-Based Access Control
//     abacEnabled: true,         // Attribute-Based Access Control
//     tokenStorage: 'localStorage'
//   }
// });
// 
// // Use React Query hooks - tokens are injected automatically!
// const loginMutation = useLogin({
//   onSuccess: () => {
//     console.log('Logged in as:', auth.getCurrentUser());
//     console.log('User role:', auth.getRole());
//     console.log('Permissions:', auth.getPermissions());
//   }
// });
// 
// // Protected endpoints work automatically - no manual token management needed!
// const { data: orders } = useGetUserOrders({ user_id: '123' });