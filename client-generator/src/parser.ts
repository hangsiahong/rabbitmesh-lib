import axios from 'axios';
import { ServiceDefinition, ServiceMethod, Parameter, TypeDefinition } from './types';

export class ServiceParser {
  constructor(private gatewayUrl: string) {}

  async parseServicesFromGateway(): Promise<ServiceDefinition[]> {
    try {
      // Fetch service registry from dynamic gateway
      const response = await axios.get(`${this.gatewayUrl}/api/services`);
      const gatewayData = response.data;
      
      // Parse the dynamic gateway response format
      if (!gatewayData.endpoints || !Array.isArray(gatewayData.endpoints)) {
        throw new Error('Invalid gateway response: missing endpoints array');
      }
      
      // Group endpoints by service
      const serviceMap = new Map<string, any[]>();
      for (const endpoint of gatewayData.endpoints) {
        const serviceName = endpoint.service;
        if (!serviceName) {
          console.error('Endpoint missing service name:', endpoint);
          continue;
        }
        if (!serviceMap.has(serviceName)) {
          serviceMap.set(serviceName, []);
        }
        serviceMap.get(serviceName)!.push(endpoint);
      }
      
      // Convert to service definitions
      const serviceDefinitions: ServiceDefinition[] = [];
      for (const [serviceName, endpoints] of serviceMap.entries()) {
        const serviceDef = this.parseServiceFromEndpoints(serviceName, endpoints);
        serviceDefinitions.push(serviceDef);
      }
      
      return serviceDefinitions;
    } catch (error) {
      throw new Error(`Failed to fetch services from gateway: ${error}`);
    }
  }

  private parseServiceFromEndpoints(serviceName: string, endpoints: any[]): ServiceDefinition {
    const methods = endpoints.map(endpoint => this.parseEndpointToMethod(endpoint));
    
    return {
      name: this.toCamelCase((serviceName || 'unknown').replace('-service', '')), // Remove -service suffix
      methods,
      version: '1.0.0',
      description: `Auto-generated from ${serviceName || 'unknown service'}`
    };
  }

  private parseEndpointToMethod(endpoint: any): ServiceMethod {
    return {
      name: this.toCamelCase(endpoint.handler),
      httpMethod: endpoint.method.toUpperCase(),
      path: endpoint.path,
      responseType: 'any', // Dynamic discovery doesn't provide detailed type info yet
      parameters: this.inferParametersFromPath(endpoint.path, endpoint.method),
      description: `${endpoint.method} ${endpoint.path}`
    };
  }

  private inferParametersFromPath(path: string, httpMethod: string): Parameter[] {
    const parameters: Parameter[] = [];
    
    // Extract path parameters (like :id, :userId, etc.)
    const pathParamMatches = path.match(/:(\w+)/g);
    if (pathParamMatches) {
      for (const match of pathParamMatches) {
        const paramName = match.substring(1); // Remove the ':'
        parameters.push({
          name: paramName,
          type: 'string',
          required: true,
          source: 'path',
          description: `Path parameter: ${paramName}`
        });
      }
    }
    
    // Add common body parameters for POST/PUT methods
    if (httpMethod === 'POST' || httpMethod === 'PUT') {
      parameters.push({
        name: 'data',
        type: 'any',
        required: true,
        source: 'body',
        description: 'Request body data'
      });
    }
    
    return parameters;
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

  // Manual service definitions for todo-app
  getManualTodoAppServices(): ServiceDefinition[] {
    return [this.getManualAuthService(), this.getManualTodoAppService(), this.getManualNotificationService()];
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
          path: '/api/v1/auth-service/register',
          requestType: 'RegisterRequest',
          responseType: 'any',
          parameters: [
            { name: 'username', type: 'string', required: true, source: 'body' },
            { name: 'email', type: 'string', required: true, source: 'body' },
            { name: 'password', type: 'string', required: true, source: 'body' }
          ]
        },
        {
          name: 'login',
          httpMethod: 'POST',
          path: '/api/v1/auth-service/login',
          requestType: 'LoginRequest',
          responseType: 'any',
          parameters: [
            { name: 'username', type: 'string', required: true, source: 'body' },
            { name: 'password', type: 'string', required: true, source: 'body' }
          ]
        },
        {
          name: 'getProfile',
          httpMethod: 'GET',
          path: '/api/v1/auth-service/get_profile',
          responseType: 'any',
          parameters: [
            { name: 'user_id', type: 'string', required: true, source: 'query' }
          ]
        },
        {
          name: 'listUsers',
          httpMethod: 'GET',
          path: '/api/v1/auth-service/list_users',
          responseType: 'any',
          parameters: []
        },
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

  getManualTodoAppService(): ServiceDefinition {
    return {
      name: 'todo',
      methods: [
        {
          name: 'createTodo',
          httpMethod: 'POST',
          path: '/api/v1/todo-service/create_todo',
          requestType: 'CreateTodoRequest',
          responseType: 'any',
          parameters: [
            { name: 'title', type: 'string', required: true, source: 'body' },
            { name: 'description', type: 'string', required: false, source: 'body' },
            { name: 'priority', type: 'string', required: false, source: 'body' }
          ]
        },
        {
          name: 'getTodos',
          httpMethod: 'GET',
          path: '/api/v1/todo-service/get_todos',
          responseType: 'any',
          parameters: []
        },
        {
          name: 'getTodo',
          httpMethod: 'GET',
          path: '/api/v1/todo-service/get_todo',
          responseType: 'any',
          parameters: [
            { name: 'todo_id', type: 'string', required: true, source: 'query' }
          ]
        },
        {
          name: 'updateTodo',
          httpMethod: 'PUT',
          path: '/api/v1/todo-service/update_todo',
          responseType: 'any',
          parameters: [
            { name: 'todo_id', type: 'string', required: true, source: 'body' },
            { name: 'title', type: 'string', required: false, source: 'body' },
            { name: 'description', type: 'string', required: false, source: 'body' },
            { name: 'completed', type: 'boolean', required: false, source: 'body' }
          ]
        },
        {
          name: 'deleteTodo',
          httpMethod: 'DELETE',
          path: '/api/v1/todo-service/delete_todo',
          responseType: 'any',
          parameters: [
            { name: 'todo_id', type: 'string', required: true, source: 'query' }
          ]
        },
        {
          name: 'getTodoStats',
          httpMethod: 'GET',
          path: '/api/v1/todo-service/get_todo_stats',
          responseType: 'any',
          parameters: []
        }
      ]
    };
  }

  getManualNotificationService(): ServiceDefinition {
    return {
      name: 'notification',
      methods: [
        {
          name: 'sendNotification',
          httpMethod: 'POST',
          path: '/api/v1/notification-service/send_notification',
          requestType: 'SendNotificationRequest',
          responseType: 'any',
          parameters: [
            { name: 'user_id', type: 'string', required: true, source: 'body' },
            { name: 'type', type: 'string', required: true, source: 'body' },
            { name: 'subject', type: 'string', required: true, source: 'body' },
            { name: 'body', type: 'string', required: true, source: 'body' }
          ]
        },
        {
          name: 'getNotificationHistory',
          httpMethod: 'GET',
          path: '/api/v1/notification-service/get_notification_history',
          responseType: 'any',
          parameters: [
            { name: 'user_id', type: 'string', required: true, source: 'query' }
          ]
        },
        {
          name: 'streamNotifications',
          httpMethod: 'GET',
          path: '/api/v1/notification-service/stream_notifications',
          responseType: 'any',
          parameters: [
            { name: 'user_id', type: 'string', required: true, source: 'query' }
          ]
        },
        {
          name: 'getNotificationStats',
          httpMethod: 'GET',
          path: '/api/v1/notification-service/get_notification_stats',
          responseType: 'any',
          parameters: []
        }
      ]
    };
  }
}