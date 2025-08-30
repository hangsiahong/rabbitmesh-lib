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
    
    // Extract path parameters (like {id}, {user_id}, etc.)
    const pathParamMatches = path.match(/\{(\w+)\}/g);
    if (pathParamMatches) {
      for (const match of pathParamMatches) {
        const paramName = match.slice(1, -1); // Remove the '{' and '}'
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

}