import axios from 'axios';
import { ServiceDefinition, ServiceMethod, Parameter, TypeDefinition } from './types';

export class ServiceParser {
  constructor(private gatewayUrl: string) {}

  async parseServicesFromGateway(): Promise<ServiceDefinition[]> {
    try {
      // Fetch service registry from gateway
      const response = await axios.get(`${this.gatewayUrl}/services`);
      const services = response.data;
      
      const serviceDefinitions: ServiceDefinition[] = [];
      
      for (const [serviceName, serviceInfo] of Object.entries(services as any)) {
        const serviceDef = this.parseService(serviceName, serviceInfo);
        serviceDefinitions.push(serviceDef);
      }
      
      return serviceDefinitions;
    } catch (error) {
      throw new Error(`Failed to fetch services from gateway: ${error}`);
    }
  }

  private parseService(name: string, serviceInfo: any): ServiceDefinition {
    const methods = serviceInfo.methods?.map((method: any) => this.parseMethod(method)) || [];
    
    return {
      name: this.toCamelCase(name),
      methods,
      version: serviceInfo.version,
      description: serviceInfo.description
    };
  }

  private parseMethod(methodInfo: any): ServiceMethod {
    const { httpMethod, path } = this.parseHttpRoute(methodInfo.http_route || '');
    
    return {
      name: this.toCamelCase(methodInfo.name),
      httpMethod,
      path,
      responseType: this.mapRustTypeToTypeScript(methodInfo.return_type || 'any'),
      parameters: methodInfo.parameters?.map((p: any) => this.parseParameter(p, path)) || [],
      description: methodInfo.description
    };
  }

  private parseParameter(paramInfo: any, path: string): Parameter {
    const isPathParam = path.includes(`:${paramInfo.name}`);
    
    return {
      name: paramInfo.name,
      type: this.mapRustTypeToTypeScript(paramInfo.param_type),
      required: paramInfo.required !== false,
      source: isPathParam ? 'path' : (paramInfo.param_type.includes('Request') ? 'body' : 'query'),
      description: paramInfo.description
    };
  }

  private parseHttpRoute(route: string): { httpMethod: string; path: string } {
    const parts = route.split(' ');
    if (parts.length !== 2) {
      return { httpMethod: 'GET', path: '/' };
    }
    
    return {
      httpMethod: parts[0].toUpperCase(),
      path: parts[1]
    };
  }

  private mapRustTypeToTypeScript(rustType: string): string {
    const typeMap: Record<string, string> = {
      'String': 'string',
      'i32': 'number',
      'i64': 'number',
      'f32': 'number',
      'f64': 'number',
      'bool': 'boolean',
      'Vec<String>': 'string[]',
      'Vec<Todo>': 'Todo[]',
      'Option<String>': 'string | null',
      'DateTime<Utc>': 'string',
      'Result<TodoResponse, String>': 'TodoResponse',
      'Result<TodoListResponse, String>': 'TodoListResponse',
      'Result<serde_json::Value, String>': 'any'
    };
    
    return typeMap[rustType] || rustType.replace(/([A-Z])/g, '$1').trim();
  }

  private toCamelCase(str: string): string {
    return str
      .replace(/[-_\s]+(.)?/g, (_, c) => (c ? c.toUpperCase() : ''))
      .replace(/^[A-Z]/, c => c.toLowerCase());
  }

  // Manual service definitions for blog platform
  getManualBlogPlatformServices(): ServiceDefinition[] {
    return [this.getManualAuthService(), this.getManualBlogService()];
  }

  getManualAuthService(): ServiceDefinition {
    return {
      name: 'auth',
      methods: [
        {
          name: 'register',
          httpMethod: 'POST',
          path: '/auth/register',
          requestType: 'RegisterRequest',
          responseType: 'AuthResponse',
          parameters: [
            { name: 'request', type: 'RegisterRequest', required: true, source: 'body' }
          ]
        },
        {
          name: 'login',
          httpMethod: 'POST',
          path: '/auth/login',
          requestType: 'LoginRequest',
          responseType: 'AuthResponse',
          parameters: [
            { name: 'request', type: 'LoginRequest', required: true, source: 'body' }
          ]
        },
        {
          name: 'validateToken',
          httpMethod: 'POST',
          path: '/auth/validate',
          requestType: 'ValidateTokenRequest',
          responseType: 'ValidateTokenResponse',
          parameters: [
            { name: 'request', type: 'ValidateTokenRequest', required: true, source: 'body' }
          ]
        },
        {
          name: 'getProfile',
          httpMethod: 'GET',
          path: '/auth/profile/:user_id',
          responseType: 'AuthResponse',
          parameters: [
            { name: 'user_id', type: 'string', required: true, source: 'path' }
          ]
        },
        {
          name: 'updateProfile',
          httpMethod: 'PUT',
          path: '/auth/profile/:user_id',
          responseType: 'AuthResponse',
          parameters: [
            { name: 'user_id', type: 'string', required: true, source: 'path' },
            { name: 'request', type: 'UpdateProfileRequest', required: true, source: 'body' }
          ]
        }
      ]
    };
  }

  getManualBlogService(): ServiceDefinition {
    return {
      name: 'blog',
      methods: [
        {
          name: 'createPost',
          httpMethod: 'POST',
          path: '/posts',
          requestType: 'CreatePostRequest',
          responseType: 'PostResponse',
          parameters: [
            { name: 'authorization', type: 'string', required: true, source: 'body' },
            { name: 'request', type: 'CreatePostRequest', required: true, source: 'body' }
          ]
        },
        {
          name: 'getPost',
          httpMethod: 'GET',
          path: '/posts/:id',
          responseType: 'PostResponse',
          parameters: [
            { name: 'id', type: 'string', required: true, source: 'path' }
          ]
        },
        {
          name: 'listPosts',
          httpMethod: 'GET',
          path: '/posts',
          responseType: 'PostListResponse',
          parameters: [
            { name: 'page', type: 'number', required: false, source: 'query' },
            { name: 'per_page', type: 'number', required: false, source: 'query' },
            { name: 'status', type: 'string', required: false, source: 'query' }
          ]
        },
        {
          name: 'updatePost',
          httpMethod: 'PUT',
          path: '/posts/:id',
          responseType: 'PostResponse',
          parameters: [
            { name: 'authorization', type: 'string', required: true, source: 'body' },
            { name: 'id', type: 'string', required: true, source: 'path' },
            { name: 'request', type: 'UpdatePostRequest', required: true, source: 'body' }
          ]
        },
        {
          name: 'deletePost',
          httpMethod: 'DELETE',
          path: '/posts/:id',
          responseType: 'PostResponse',
          parameters: [
            { name: 'authorization', type: 'string', required: true, source: 'body' },
            { name: 'id', type: 'string', required: true, source: 'path' }
          ]
        },
        {
          name: 'createComment',
          httpMethod: 'POST',
          path: '/posts/:post_id/comments',
          responseType: 'CommentResponse',
          parameters: [
            { name: 'authorization', type: 'string', required: true, source: 'body' },
            { name: 'post_id', type: 'string', required: true, source: 'path' },
            { name: 'request', type: 'CreateCommentRequest', required: true, source: 'body' }
          ]
        },
        {
          name: 'getComments',
          httpMethod: 'GET',
          path: '/posts/:post_id/comments',
          responseType: 'CommentListResponse',
          parameters: [
            { name: 'post_id', type: 'string', required: true, source: 'path' }
          ]
        }
      ]
    };
  }

  // Manual service definition for TodoService as fallback
  getManualTodoService(): ServiceDefinition {
    return {
      name: 'todo',
      methods: [
        {
          name: 'createTodo',
          httpMethod: 'POST',
          path: '/todos',
          requestType: 'CreateTodoRequest',
          responseType: 'TodoResponse',
          parameters: [
            { name: 'request', type: 'CreateTodoRequest', required: true, source: 'body' }
          ]
        },
        {
          name: 'getTodo',
          httpMethod: 'GET',
          path: '/todos/:id',
          responseType: 'TodoResponse',
          parameters: [
            { name: 'id', type: 'string', required: true, source: 'path' }
          ]
        },
        {
          name: 'listTodos',
          httpMethod: 'GET',
          path: '/todos',
          responseType: 'TodoListResponse',
          parameters: []
        },
        {
          name: 'updateTodo',
          httpMethod: 'PUT',
          path: '/todos/:id',
          responseType: 'TodoResponse',
          parameters: [
            { name: 'id', type: 'string', required: true, source: 'path' },
            { name: 'request', type: 'UpdateTodoRequest', required: true, source: 'body' }
          ]
        },
        {
          name: 'deleteTodo',
          httpMethod: 'DELETE',
          path: '/todos/:id',
          responseType: 'TodoResponse',
          parameters: [
            { name: 'id', type: 'string', required: true, source: 'path' }
          ]
        },
        {
          name: 'completeTodo',
          httpMethod: 'POST',
          path: '/todos/:id/complete',
          responseType: 'TodoResponse',
          parameters: [
            { name: 'id', type: 'string', required: true, source: 'path' }
          ]
        },
        {
          name: 'getStats',
          httpMethod: 'GET',
          path: '/todos/stats',
          responseType: 'any',
          parameters: []
        }
      ]
    };
  }
}