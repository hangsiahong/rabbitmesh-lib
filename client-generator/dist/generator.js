"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.ClientGenerator = void 0;
const fs_1 = __importDefault(require("fs"));
const path_1 = __importDefault(require("path"));
class ClientGenerator {
    constructor(config) {
        this.config = config;
    }
    async generateClient(services) {
        // Create output directory
        if (!fs_1.default.existsSync(this.config.outputDir)) {
            fs_1.default.mkdirSync(this.config.outputDir, { recursive: true });
        }
        // Generate types file
        await this.generateTypes(services);
        // Generate service clients
        await this.generateServiceClients(services);
        // Generate main client class
        await this.generateMainClient(services);
        // Generate authentication manager
        await this.generateAuthManager();
        // Generate package.json
        await this.generatePackageJson();
        // Generate tsconfig.json
        await this.generateTsConfig();
        // Generate index.ts
        await this.generateIndex(services);
    }
    async generateTypes(services) {
        const types = this.extractTypes(services);
        const content = this.generateTypesContent(types);
        fs_1.default.writeFileSync(path_1.default.join(this.config.outputDir, 'types.ts'), content);
    }
    extractTypes(services) {
        const types = [];
        // Dynamically extract types from service schemas
        for (const service of services) {
            for (const method of service.methods) {
                // Extract request types from method parameters
                if (method.parameters && method.parameters.length > 0) {
                    this.extractRequestType(types, method);
                }
                // Extract response types - keep it generic, no hardcoded patterns
                this.extractResponseType(types, method, service.name);
            }
        }
        // Add common generic types
        this.addGenericTypes(types);
        return types;
    }
    extractRequestType(types, method) {
        const requestTypeName = this.getRequestTypeName(method.name);
        // Check if type already exists
        if (types.some(t => t.name === requestTypeName)) {
            return;
        }
        // Extract fields from actual method parameters only
        const fields = method.parameters
            .filter(p => p.source === 'body' || p.source === 'query')
            .map(p => ({
            name: p.name,
            type: this.mapParameterType(p.type),
            optional: !p.required,
            description: p.description || `Parameter: ${p.name}`
        }));
        if (fields.length > 0) {
            types.push({
                name: requestTypeName,
                fields,
                description: `Request type for ${method.name}`
            });
        }
    }
    extractResponseType(types, method, serviceName) {
        const responseTypeName = this.getResponseTypeName(method.name, serviceName);
        // Check if type already exists
        if (types.some(t => t.name === responseTypeName)) {
            return;
        }
        // Generic response type - no hardcoded patterns
        types.push({
            name: responseTypeName,
            fields: [
                { name: 'data', type: 'any', optional: true, description: 'Response data' },
                { name: 'success', type: 'boolean', optional: true, description: 'Operation success status' },
                { name: 'message', type: 'string', optional: true, description: 'Response message' }
            ],
            description: `Response type for ${method.name}`
        });
    }
    addGenericTypes(types) {
        types.push({
            name: 'ApiResponse',
            fields: [
                { name: 'data', type: 'any', optional: true },
                { name: 'success', type: 'boolean', optional: true },
                { name: 'message', type: 'string', optional: true }
            ]
        });
    }
    getRequestTypeName(methodName) {
        return methodName.charAt(0).toUpperCase() + methodName.slice(1) + 'Request';
    }
    getResponseTypeName(methodName, serviceName) {
        return methodName.charAt(0).toUpperCase() + methodName.slice(1) + 'Response';
    }
    mapParameterType(type) {
        switch (type.toLowerCase()) {
            case 'string': return 'string';
            case 'number':
            case 'int':
            case 'integer': return 'number';
            case 'bool':
            case 'boolean': return 'boolean';
            case 'array': return 'any[]';
            default: return 'any';
        }
    }
    generateTypesContent(types) {
        const typeDefinitions = types.map(type => {
            const fields = type.fields.map(field => {
                const optional = field.optional ? '?' : '';
                return `  ${field.name}${optional}: ${field.type};`;
            }).join('\n');
            const description = type.description ? `/** ${type.description} */\n` : '';
            return `${description}export interface ${type.name} {\n${fields}\n}`;
        }).join('\n\n');
        return `// Auto-generated types for RabbitMesh services\n// Do not edit this file manually\n\n${typeDefinitions}\n`;
    }
    async generateServiceClients(services) {
        for (const service of services) {
            const content = this.generateServiceClient(service);
            const filename = `${service.name}Client.ts`;
            fs_1.default.writeFileSync(path_1.default.join(this.config.outputDir, filename), content);
        }
    }
    generateServiceClient(service) {
        const queryHooks = service.methods
            .filter(method => method.httpMethod === 'GET')
            .map(method => this.generateQueryHook(method, service.name))
            .join('\n\n');
        const mutationHooks = service.methods
            .filter(method => ['POST', 'PUT', 'DELETE'].includes(method.httpMethod))
            .map(method => this.generateMutationHook(method, service.name))
            .join('\n\n');
        return `import { useQuery, useMutation, UseQueryOptions, UseMutationOptions } from '@tanstack/react-query';
import axios from 'axios';
import * as Types from './types';

// Query Keys
export const ${service.name}Keys = {
${service.methods.filter(m => m.httpMethod === 'GET').map(method => `  ${method.name}: (${this.getQueryKeyParams(method)}) => ['${service.name}', '${method.name}'${this.getQueryKeyParamsUsage(method)}] as const,`).join('\n')}
} as const;

// Query Hooks
${queryHooks}

// Mutation Hooks
${mutationHooks}`;
    }
    generateQueryHook(method, serviceName) {
        const hookName = `use${this.capitalize(method.name)}`;
        const pathParams = method.parameters.filter(p => p.source === 'path');
        const queryParams = method.parameters.filter(p => p.source === 'query');
        // Build path with parameter substitution
        let path = method.path;
        pathParams.forEach(param => {
            path = path.replace(`{${param.name}}`, `\${${param.name}}`);
        });
        const params = this.generateHookParams(method);
        const responseType = this.getResponseTypeName(method.name, serviceName);
        return `export function ${hookName}(
  ${params}
  options?: Omit<UseQueryOptions<Types.${responseType}>, 'queryKey' | 'queryFn'>
) {
  return useQuery({
    queryKey: ${serviceName}Keys.${method.name}(${this.getQueryKeyParamsUsage(method).slice(2)}),
    queryFn: async () => {
      const response = await axios.get(\`${path}\`${this.buildQueryConfig(queryParams)});
      return response.data;
    },
    ...options,
  });
}`;
    }
    generateMutationHook(method, serviceName) {
        const hookName = `use${this.capitalize(method.name)}`;
        const pathParams = method.parameters.filter(p => p.source === 'path');
        const bodyParam = method.parameters.find(p => p.source === 'body');
        // Build path with parameter substitution
        let path = method.path;
        pathParams.forEach(param => {
            path = path.replace(`{${param.name}}`, `\${${param.name}}`);
        });
        const variablesType = this.getMutationVariablesType(method);
        const responseType = this.getResponseTypeName(method.name, serviceName);
        return `export function ${hookName}(
  options?: UseMutationOptions<Types.${responseType}, Error, ${variablesType}>
) {
  return useMutation({
    mutationFn: async (variables: ${variablesType}) => {
      ${this.generateMutationBody(method, path)}
    },
    ...options,
  });
}`;
    }
    generateMutationBody(method, path) {
        const pathParams = method.parameters.filter(p => p.source === 'path');
        const bodyParam = method.parameters.find(p => p.source === 'body');
        // Extract path parameters from variables
        const pathExtraction = pathParams.map(p => `const ${p.name} = variables.${p.name};`).join('\n      ');
        const httpMethod = method.httpMethod.toLowerCase();
        const bodyArg = bodyParam ? ', variables.data' : '';
        return `${pathExtraction}
      const response = await axios.${httpMethod}(\`${path}\`${bodyArg});
      return response.data;`;
    }
    getMutationVariablesType(method) {
        const pathParams = method.parameters.filter(p => p.source === 'path');
        const bodyParam = method.parameters.find(p => p.source === 'body');
        const pathFields = pathParams.map(p => `${p.name}: string`);
        const bodyFields = bodyParam ? ['data: any'] : [];
        const allFields = [...pathFields, ...bodyFields];
        if (allFields.length === 0) {
            return 'void';
        }
        return `{ ${allFields.join('; ')} }`;
    }
    generateHookParams(method) {
        const pathParams = method.parameters.filter(p => p.source === 'path');
        const queryParams = method.parameters.filter(p => p.source === 'query');
        const allParams = [...pathParams, ...queryParams];
        if (allParams.length === 0) {
            return '';
        }
        const paramList = allParams.map(p => {
            const optional = p.required ? '' : '?';
            return `${p.name}${optional}: ${this.mapParameterType(p.type)}`;
        });
        return paramList.join(', ') + ',\n  ';
    }
    getQueryKeyParams(method) {
        const pathParams = method.parameters.filter(p => p.source === 'path');
        const queryParams = method.parameters.filter(p => p.source === 'query');
        const allParams = [...pathParams, ...queryParams];
        if (allParams.length === 0) {
            return '';
        }
        return allParams.map(p => {
            const optional = p.required ? '' : '?';
            return `${p.name}${optional}: ${this.mapParameterType(p.type)}`;
        }).join(', ');
    }
    getQueryKeyParamsUsage(method) {
        const pathParams = method.parameters.filter(p => p.source === 'path');
        const queryParams = method.parameters.filter(p => p.source === 'query');
        const allParams = [...pathParams, ...queryParams];
        if (allParams.length === 0) {
            return '';
        }
        return ', ' + allParams.map(p => p.name).join(', ');
    }
    buildQueryConfig(queryParams) {
        if (queryParams.length === 0) {
            return '';
        }
        const paramObj = queryParams.map(p => `${p.name}`).join(', ');
        return `, { params: { ${paramObj} } }`;
    }
    generateMethodParams(method) {
        if (method.parameters.length === 0)
            return '';
        const params = method.parameters.map(param => {
            const optional = param.required ? '' : '?';
            return `${param.name}${optional}: ${param.type}`;
        });
        return params.join(', ');
    }
    async generateMainClient(services) {
        const serviceImports = services.map(service => `export * from './${service.name}Client';`).join('\n');
        const content = `// Re-export all service hooks
${serviceImports}

// Export authentication system
export * from './auth-manager';

// Axios configuration for the generated hooks
import axios, { AxiosRequestConfig, AxiosResponse } from 'axios';
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
    async (config: AxiosRequestConfig): Promise<AxiosRequestConfig> => {
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
// const { data: orders } = useGetUserOrders({ user_id: '123' });`;
        fs_1.default.writeFileSync(path_1.default.join(this.config.outputDir, 'client.ts'), content);
    }
    async generateAuthManager() {
        const content = `// Comprehensive Authentication Manager for RabbitMesh
// Supports JWT, RBAC, ABAC automatically
import axios, { AxiosRequestConfig, AxiosResponse, AxiosError } from 'axios';

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
  public async injectAuthHeaders(config: AxiosRequestConfig): Promise<AxiosRequestConfig> {
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
        config.headers['Authorization'] = \`Bearer \${this.tokenInfo.access_token}\`;
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
    if (error.response?.status === 401 && this.tokenInfo?.refresh_token && !originalRequest?._retry) {
      originalRequest._retry = true;
      
      try {
        const newToken = await this.refreshAccessToken();
        
        // Retry original request with new token
        if (originalRequest) {
          originalRequest.headers = originalRequest.headers || {};
          originalRequest.headers['Authorization'] = \`Bearer \${newToken}\`;
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

    const policyKey = \`\${resource}:\${action}\`;
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
}
`;
        fs_1.default.writeFileSync(path_1.default.join(this.config.outputDir, 'auth-manager.ts'), content);
    }
    async generatePackageJson() {
        const packageJson = {
            name: this.config.packageName,
            version: '1.0.0',
            description: 'Auto-generated React Query hooks for RabbitMesh services',
            main: 'index.js',
            types: 'index.d.ts',
            scripts: {
                build: 'tsc'
            },
            dependencies: {
                axios: '^1.6.0',
                '@tanstack/react-query': '^5.0.0'
            },
            peerDependencies: {
                react: '>=16.8.0'
            },
            devDependencies: {
                typescript: '^5.0.0',
                '@types/node': '^20.0.0',
                '@types/react': '^18.0.0'
            },
            keywords: ['rabbitmesh', 'microservices', 'react-query', 'hooks', 'typescript'],
            author: 'RabbitMesh Client Generator',
            license: 'MIT'
        };
        fs_1.default.writeFileSync(path_1.default.join(this.config.outputDir, 'package.json'), JSON.stringify(packageJson, null, 2));
    }
    async generateTsConfig() {
        const tsConfig = {
            compilerOptions: {
                target: 'ES2018',
                module: 'commonjs',
                outDir: './dist',
                rootDir: './',
                strict: true,
                esModuleInterop: true,
                skipLibCheck: true,
                forceConsistentCasingInFileNames: true,
                declaration: true,
                declarationDir: './dist'
            },
            include: ['*.ts'],
            exclude: ['node_modules', 'dist']
        };
        fs_1.default.writeFileSync(path_1.default.join(this.config.outputDir, 'tsconfig.json'), JSON.stringify(tsConfig, null, 2));
    }
    async generateIndex(services) {
        const exports = [
            "// Re-export all hooks and utilities",
            "export * from './client';",
            "export * from './types';",
            ...services.map(service => `export * from './${service.name}Client';`)
        ].join('\n');
        fs_1.default.writeFileSync(path_1.default.join(this.config.outputDir, 'index.ts'), exports + '\n');
    }
    capitalize(str) {
        return str.charAt(0).toUpperCase() + str.slice(1);
    }
}
exports.ClientGenerator = ClientGenerator;
//# sourceMappingURL=generator.js.map