// Comprehensive Authentication Manager for RabbitMesh
// Supports JWT, RBAC, ABAC automatically
import axios, { AxiosRequestConfig, AxiosResponse, AxiosError, InternalAxiosRequestConfig } from 'axios';

export interface AuthConfig {
  autoManageTokens: boolean;
  rbacEnabled: boolean;
  abacEnabled: boolean;
  tokenStorage: 'localStorage' | 'sessionStorage' | 'memory';
  refreshThreshold: number;
}

export interface UserInfo {
  id: string;
  email: string;
  name: string;
  role: string;
  permissions: string[];
  attributes?: Record<string, any>;
}

export interface LoginResponse {
  access_token: string;
  refresh_token: string;
  expires_in: number;
  user: UserInfo;
}

export interface TokenInfo {
  access_token: string;
  refresh_token: string;
  expires_at: Date;
  user: UserInfo;
}

export class AuthManager {
  private config: AuthConfig;
  private tokenInfo: TokenInfo | null = null;
  private refreshPromise: Promise<string> | null = null;
  
  // In-memory storage for when localStorage/sessionStorage isn't available
  private memoryStorage: Record<string, string> = {};

  constructor(config: AuthConfig) {
    this.config = config;
    
    if (config.autoManageTokens) {
      this.loadStoredTokens();
    }
  }

  /**
   * Load tokens from storage on initialization
   */
  private loadStoredTokens(): void {
    try {
      const storedTokens = this.getFromStorage('rabbitmesh_tokens');
      if (storedTokens) {
        const parsed = JSON.parse(storedTokens);
        parsed.expires_at = new Date(parsed.expires_at);
        this.tokenInfo = parsed;
        
        // Check if token is expired
        if (this.isTokenExpired()) {
          this.clearTokens();
        }
      }
    } catch (error) {
      console.warn('Failed to load stored tokens:', error);
      this.clearTokens();
    }
  }

  /**
   * Store tokens securely
   */
  private storeTokens(tokenInfo: TokenInfo): void {
    this.tokenInfo = tokenInfo;
    
    if (this.config.autoManageTokens) {
      try {
        this.setInStorage('rabbitmesh_tokens', JSON.stringify(tokenInfo));
      } catch (error) {
        console.warn('Failed to store tokens:', error);
      }
    }
  }

  /**
   * Clear all stored tokens
   */
  public clearTokens(): void {
    this.tokenInfo = null;
    this.removeFromStorage('rabbitmesh_tokens');
  }

  /**
   * Storage abstraction - works with localStorage, sessionStorage, or memory
   */
  private getFromStorage(key: string): string | null {
    try {
      switch (this.config.tokenStorage) {
        case 'localStorage':
          return typeof window !== 'undefined' ? window.localStorage.getItem(key) : null;
        case 'sessionStorage':
          return typeof window !== 'undefined' ? window.sessionStorage.getItem(key) : null;
        case 'memory':
          return this.memoryStorage[key] || null;
        default:
          return null;
      }
    } catch {
      return null;
    }
  }

  private setInStorage(key: string, value: string): void {
    try {
      switch (this.config.tokenStorage) {
        case 'localStorage':
          if (typeof window !== 'undefined') {
            window.localStorage.setItem(key, value);
          }
          break;
        case 'sessionStorage':
          if (typeof window !== 'undefined') {
            window.sessionStorage.setItem(key, value);
          }
          break;
        case 'memory':
          this.memoryStorage[key] = value;
          break;
      }
    } catch (error) {
      console.warn('Failed to store in storage:', error);
    }
  }

  private removeFromStorage(key: string): void {
    try {
      switch (this.config.tokenStorage) {
        case 'localStorage':
          if (typeof window !== 'undefined') {
            window.localStorage.removeItem(key);
          }
          break;
        case 'sessionStorage':
          if (typeof window !== 'undefined') {
            window.sessionStorage.removeItem(key);
          }
          break;
        case 'memory':
          delete this.memoryStorage[key];
          break;
      }
    } catch (error) {
      console.warn('Failed to remove from storage:', error);
    }
  }

  /**
   * Check if current token is expired or about to expire
   */
  private isTokenExpired(): boolean {
    if (!this.tokenInfo) return true;
    
    const now = new Date();
    const threshold = this.config.refreshThreshold * 60 * 1000; // Convert minutes to milliseconds
    
    return (this.tokenInfo.expires_at.getTime() - threshold) <= now.getTime();
  }

  /**
   * Inject authentication headers into axios request
   */
  public async injectAuthHeaders(config: InternalAxiosRequestConfig): Promise<InternalAxiosRequestConfig> {
    // Skip auth injection for auth endpoints to avoid circular calls
    if (config.url?.includes('/auth/login') || 
        config.url?.includes('/auth/refresh') || 
        config.url?.includes('/auth/validate')) {
      return config;
    }

    if (this.config.autoManageTokens && this.tokenInfo) {
      // Check if token needs refresh
      if (this.isTokenExpired()) {
        try {
          await this.refreshAccessToken();
        } catch (error) {
          console.warn('Failed to refresh token, clearing auth state:', error);
          this.clearTokens();
          return config;
        }
      }

      // Inject Authorization header
      if (this.tokenInfo) {
        config.headers = config.headers || {};
        config.headers['Authorization'] = `Bearer ${this.tokenInfo.access_token}`;
      }
    }

    return config;
  }

  /**
   * Handle authentication responses (e.g., from login)
   */
  public handleAuthResponse(response: AxiosResponse): void {
    // Auto-detect login response and store tokens
    if (response.config.url?.includes('/auth/login') && response.data) {
      const loginData = response.data as LoginResponse;
      
      if (loginData.access_token && loginData.user) {
        const expiresAt = new Date();
        expiresAt.setSeconds(expiresAt.getSeconds() + loginData.expires_in);
        
        this.storeTokens({
          access_token: loginData.access_token,
          refresh_token: loginData.refresh_token,
          expires_at: expiresAt,
          user: loginData.user,
        });
        
        console.log('🔐 Authentication successful:', {
          user: loginData.user.email,
          role: loginData.user.role,
          permissions: loginData.user.permissions?.length || 0,
          expires: expiresAt.toISOString(),
        });
      }
    }
  }

  /**
   * Handle authentication errors (401, 403, token refresh)
   */
  public async handleAuthError(error: AxiosError): Promise<any> {
    const originalRequest = error.config;
    
    // Handle 401 Unauthorized - try to refresh token
    if (error.response?.status === 401 && this.tokenInfo?.refresh_token && originalRequest && !originalRequest._retry) {
      originalRequest._retry = true;
      
      try {
        const newToken = await this.refreshAccessToken();
        
        // Retry original request with new token
        if (originalRequest) {
          originalRequest.headers = originalRequest.headers || {};
          originalRequest.headers['Authorization'] = `Bearer ${newToken}`;
          return axios(originalRequest);
        }
      } catch (refreshError) {
        console.warn('Token refresh failed, clearing auth state');
        this.clearTokens();
        return Promise.reject(error);
      }
    }
    
    // Handle 403 Forbidden - insufficient permissions
    if (error.response?.status === 403) {
      console.warn('Access denied - insufficient permissions:', {
        url: originalRequest?.url,
        userRole: this.getRole(),
        userPermissions: this.getPermissions(),
      });
    }

    return Promise.reject(error);
  }

  /**
   * Refresh the access token using the refresh token
   */
  public async refreshAccessToken(): Promise<string> {
    if (!this.tokenInfo?.refresh_token) {
      throw new Error('No refresh token available');
    }

    // Prevent multiple simultaneous refresh requests
    if (this.refreshPromise) {
      return this.refreshPromise;
    }

    this.refreshPromise = (async () => {
      try {
        const response = await axios.post('/api/v1/auth-service/auth/refresh', {
          refresh_token: this.tokenInfo!.refresh_token,
        });

        const refreshData = response.data as LoginResponse;
        
        if (refreshData.access_token) {
          const expiresAt = new Date();
          expiresAt.setSeconds(expiresAt.getSeconds() + refreshData.expires_in);
          
          this.storeTokens({
            access_token: refreshData.access_token,
            refresh_token: refreshData.refresh_token,
            expires_at: expiresAt,
            user: refreshData.user || this.tokenInfo!.user,
          });
          
          return refreshData.access_token;
        } else {
          throw new Error('Invalid refresh response');
        }
      } finally {
        this.refreshPromise = null;
      }
    })();

    return this.refreshPromise;
  }

  // Public API methods

  public isAuthenticated(): boolean {
    return this.tokenInfo !== null && !this.isTokenExpired();
  }

  public getCurrentUser(): UserInfo | null {
    return this.tokenInfo?.user || null;
  }

  public getAccessToken(): string | null {
    return this.tokenInfo?.access_token || null;
  }

  // RBAC (Role-Based Access Control) methods

  public getRole(): string | null {
    return this.tokenInfo?.user?.role || null;
  }

  public getPermissions(): string[] {
    return this.tokenInfo?.user?.permissions || [];
  }

  public hasRole(role: string): boolean {
    if (!this.config.rbacEnabled) return true;
    return this.getRole() === role;
  }

  public hasPermission(permission: string): boolean {
    if (!this.config.rbacEnabled) return true;
    return this.getPermissions().includes(permission);
  }

  public hasAnyPermission(permissions: string[]): boolean {
    if (!this.config.rbacEnabled) return true;
    const userPermissions = this.getPermissions();
    return permissions.some(p => userPermissions.includes(p));
  }

  public hasAllPermissions(permissions: string[]): boolean {
    if (!this.config.rbacEnabled) return true;
    const userPermissions = this.getPermissions();
    return permissions.every(p => userPermissions.includes(p));
  }

  // ABAC (Attribute-Based Access Control) methods

  public getAttributes(): Record<string, any> {
    return this.tokenInfo?.user?.attributes || {};
  }

  public getAttribute(key: string): any {
    return this.getAttributes()[key];
  }

  public checkAbacPolicy(resource: string, action: string, context?: Record<string, any>): boolean {
    if (!this.config.abacEnabled) return true;
    
    const user = this.getCurrentUser();
    if (!user) return false;

    // Basic ABAC policy evaluation
    // In a real implementation, this would call the policy evaluation service
    const userAttributes = this.getAttributes();
    const userContext = {
      ...userAttributes,
      role: user.role,
      permissions: user.permissions,
      ...context,
    };

    // Example policies (can be extended)
    const policies: Record<string, (ctx: any) => boolean> = {
      'users:read': (ctx) => ctx.permissions?.includes('users:read') || ctx.role === 'admin',
      'users:write': (ctx) => ctx.permissions?.includes('users:write') || ctx.role === 'admin',
      'users:delete': (ctx) => ctx.role === 'admin',
      'orders:read': (ctx) => ctx.permissions?.includes('orders:read') || ctx.role === 'admin',
      'orders:write': (ctx) => ctx.permissions?.includes('orders:write') || ctx.role === 'admin',
      'orders:delete': (ctx) => ctx.role === 'admin',
    };

    const policyKey = `${resource}:${action}`;
    const policy = policies[policyKey];
    
    if (policy) {
      return policy(userContext);
    }
    
    // Default: check if user has the required permission
    return this.hasPermission(policyKey);
  }

  /**
   * Manual logout - clears all auth state
   */
  public async logout(): Promise<void> {
    try {
      // Call logout endpoint if authenticated
      if (this.isAuthenticated()) {
        await axios.post('/api/v1/auth-service/auth/logout');
      }
    } catch (error) {
      console.warn('Logout request failed:', error);
    } finally {
      this.clearTokens();
    }
  }
}

// Axios request config extension for retry handling
declare module 'axios' {
  interface AxiosRequestConfig {
    _retry?: boolean;
  }
  interface InternalAxiosRequestConfig {
    _retry?: boolean;
  }
}
