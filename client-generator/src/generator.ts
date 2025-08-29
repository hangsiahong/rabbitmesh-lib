import fs from 'fs';
import path from 'path';
import { ServiceDefinition, ServiceMethod, TypeDefinition, GeneratorConfig } from './types';

export class ClientGenerator {
  constructor(private config: GeneratorConfig) {}

  async generateClient(services: ServiceDefinition[]): Promise<void> {
    // Create output directory
    if (!fs.existsSync(this.config.outputDir)) {
      fs.mkdirSync(this.config.outputDir, { recursive: true });
    }

    // Generate types file
    await this.generateTypes(services);
    
    // Generate service clients
    await this.generateServiceClients(services);
    
    // Generate main client class
    await this.generateMainClient(services);
    
    // Generate package.json
    await this.generatePackageJson();
    
    // Generate tsconfig.json
    await this.generateTsConfig();
    
    // Generate index.ts
    await this.generateIndex(services);
  }

  private async generateTypes(services: ServiceDefinition[]): Promise<void> {
    const types = this.extractTypes(services);
    const content = this.generateTypesContent(types);
    
    fs.writeFileSync(path.join(this.config.outputDir, 'types.ts'), content);
  }

  private extractTypes(services: ServiceDefinition[]): TypeDefinition[] {
    const types: TypeDefinition[] = [];
    
    // Check if this is a blog platform generation
    const hasAuthService = services.some(s => s.name === 'auth');
    const hasBlogService = services.some(s => s.name === 'blog');
    
    if (hasAuthService || hasBlogService) {
      // Add blog platform types
      this.addBlogPlatformTypes(types);
    } else {
      // Add Todo service types (backward compatibility)
      this.addTodoTypes(types);
    }

    return types;
  }

  private addBlogPlatformTypes(types: TypeDefinition[]): void {
    // Auth service types
    types.push({
      name: 'UserRole',
      fields: [
        { name: 'Admin', type: '"admin"', optional: false },
        { name: 'Editor', type: '"editor"', optional: false },
        { name: 'Author', type: '"author"', optional: false },
        { name: 'Reader', type: '"reader"', optional: false }
      ]
    });

    types.push({
      name: 'UserInfo',
      fields: [
        { name: 'id', type: 'string', optional: false },
        { name: 'username', type: 'string', optional: false },
        { name: 'email', type: 'string', optional: false },
        { name: 'full_name', type: 'string | null', optional: true },
        { name: 'avatar_url', type: 'string | null', optional: true },
        { name: 'role', type: 'UserRole', optional: false },
        { name: 'created_at', type: 'string', optional: false }
      ]
    });

    types.push({
      name: 'RegisterRequest',
      fields: [
        { name: 'username', type: 'string', optional: false },
        { name: 'email', type: 'string', optional: false },
        { name: 'password', type: 'string', optional: false },
        { name: 'full_name', type: 'string', optional: true }
      ]
    });

    types.push({
      name: 'LoginRequest',
      fields: [
        { name: 'email', type: 'string', optional: false },
        { name: 'password', type: 'string', optional: false }
      ]
    });

    types.push({
      name: 'AuthResponse',
      fields: [
        { name: 'success', type: 'boolean', optional: false },
        { name: 'message', type: 'string', optional: false },
        { name: 'user', type: 'UserInfo | null', optional: true },
        { name: 'token', type: 'string | null', optional: true }
      ]
    });

    types.push({
      name: 'ValidateTokenRequest',
      fields: [
        { name: 'token', type: 'string', optional: false }
      ]
    });

    types.push({
      name: 'ValidateTokenResponse',
      fields: [
        { name: 'valid', type: 'boolean', optional: false },
        { name: 'user_id', type: 'string | null', optional: true },
        { name: 'role', type: 'UserRole | null', optional: true },
        { name: 'username', type: 'string | null', optional: true }
      ]
    });

    types.push({
      name: 'UpdateProfileRequest',
      fields: [
        { name: 'full_name', type: 'string', optional: true },
        { name: 'avatar_url', type: 'string', optional: true }
      ]
    });

    // Blog service types
    types.push({
      name: 'PostStatus',
      fields: [
        { name: 'Draft', type: '"draft"', optional: false },
        { name: 'Published', type: '"published"', optional: false },
        { name: 'Archived', type: '"archived"', optional: false }
      ]
    });

    types.push({
      name: 'BlogPost',
      fields: [
        { name: 'id', type: 'string', optional: false },
        { name: 'title', type: 'string', optional: false },
        { name: 'content', type: 'string', optional: false },
        { name: 'excerpt', type: 'string', optional: false },
        { name: 'author_id', type: 'string', optional: false },
        { name: 'author_name', type: 'string', optional: false },
        { name: 'slug', type: 'string', optional: false },
        { name: 'tags', type: 'string[]', optional: false },
        { name: 'status', type: 'PostStatus', optional: false },
        { name: 'view_count', type: 'number', optional: false },
        { name: 'created_at', type: 'string', optional: false },
        { name: 'updated_at', type: 'string', optional: false },
        { name: 'published_at', type: 'string | null', optional: true }
      ]
    });

    types.push({
      name: 'CreatePostRequest',
      fields: [
        { name: 'title', type: 'string', optional: false },
        { name: 'content', type: 'string', optional: false },
        { name: 'tags', type: 'string[]', optional: false },
        { name: 'status', type: 'PostStatus', optional: false }
      ]
    });

    types.push({
      name: 'UpdatePostRequest',
      fields: [
        { name: 'title', type: 'string', optional: true },
        { name: 'content', type: 'string', optional: true },
        { name: 'tags', type: 'string[]', optional: true },
        { name: 'status', type: 'PostStatus', optional: true }
      ]
    });

    types.push({
      name: 'PostResponse',
      fields: [
        { name: 'success', type: 'boolean', optional: false },
        { name: 'message', type: 'string', optional: false },
        { name: 'post', type: 'BlogPost | null', optional: true }
      ]
    });

    types.push({
      name: 'PostListResponse',
      fields: [
        { name: 'success', type: 'boolean', optional: false },
        { name: 'message', type: 'string', optional: false },
        { name: 'posts', type: 'BlogPost[]', optional: false },
        { name: 'total', type: 'number', optional: false },
        { name: 'page', type: 'number', optional: false },
        { name: 'per_page', type: 'number', optional: false }
      ]
    });

    types.push({
      name: 'Comment',
      fields: [
        { name: 'id', type: 'string', optional: false },
        { name: 'post_id', type: 'string', optional: false },
        { name: 'author_id', type: 'string', optional: false },
        { name: 'author_name', type: 'string', optional: false },
        { name: 'content', type: 'string', optional: false },
        { name: 'parent_id', type: 'string | null', optional: true },
        { name: 'created_at', type: 'string', optional: false },
        { name: 'updated_at', type: 'string', optional: false }
      ]
    });

    types.push({
      name: 'CreateCommentRequest',
      fields: [
        { name: 'post_id', type: 'string', optional: false },
        { name: 'content', type: 'string', optional: false },
        { name: 'parent_id', type: 'string | null', optional: true }
      ]
    });

    types.push({
      name: 'CommentResponse',
      fields: [
        { name: 'success', type: 'boolean', optional: false },
        { name: 'message', type: 'string', optional: false },
        { name: 'comment', type: 'Comment | null', optional: true }
      ]
    });

    types.push({
      name: 'CommentListResponse',
      fields: [
        { name: 'success', type: 'boolean', optional: false },
        { name: 'message', type: 'string', optional: false },
        { name: 'comments', type: 'Comment[]', optional: false },
        { name: 'total', type: 'number', optional: false }
      ]
    });
  }

  private addTodoTypes(types: TypeDefinition[]): void {
    // Add common types for Todo service
    types.push({
      name: 'Todo',
      fields: [
        { name: 'id', type: 'string', optional: false },
        { name: 'title', type: 'string', optional: false },
        { name: 'description', type: 'string | null', optional: true },
        { name: 'completed', type: 'boolean', optional: false },
        { name: 'created_at', type: 'string', optional: false },
        { name: 'updated_at', type: 'string', optional: false }
      ]
    });

    types.push({
      name: 'CreateTodoRequest',
      fields: [
        { name: 'title', type: 'string', optional: false },
        { name: 'description', type: 'string', optional: true }
      ]
    });

    types.push({
      name: 'UpdateTodoRequest',
      fields: [
        { name: 'title', type: 'string', optional: true },
        { name: 'description', type: 'string', optional: true },
        { name: 'completed', type: 'boolean', optional: true }
      ]
    });

    types.push({
      name: 'TodoResponse',
      fields: [
        { name: 'success', type: 'boolean', optional: false },
        { name: 'message', type: 'string', optional: false },
        { name: 'todo', type: 'Todo | null', optional: true }
      ]
    });

    types.push({
      name: 'TodoListResponse',
      fields: [
        { name: 'success', type: 'boolean', optional: false },
        { name: 'message', type: 'string', optional: false },
        { name: 'todos', type: 'Todo[]', optional: false },
        { name: 'total', type: 'number', optional: false }
      ]
    });
  }

  private generateTypesContent(types: TypeDefinition[]): string {
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

  private async generateServiceClients(services: ServiceDefinition[]): Promise<void> {
    for (const service of services) {
      const content = this.generateServiceClient(service);
      const filename = `${service.name}Client.ts`;
      fs.writeFileSync(path.join(this.config.outputDir, filename), content);
    }
  }

  private generateServiceClient(service: ServiceDefinition): string {
    const methods = service.methods.map(method => this.generateMethod(method)).join('\n\n');
    
    return `import axios, { AxiosInstance } from 'axios';
import * as Types from './types';

export class ${this.capitalize(service.name)}Client {
  constructor(private http: AxiosInstance) {}

${methods}
}`;
  }

  private generateMethod(method: ServiceMethod): string {
    const params = this.generateMethodParams(method);
    const pathParams = method.parameters.filter(p => p.source === 'path');
    const bodyParam = method.parameters.find(p => p.source === 'body');
    const queryParams = method.parameters.filter(p => p.source === 'query');

    // Build path with parameter substitution
    let path = method.path;
    pathParams.forEach(param => {
      path = path.replace(`:${param.name}`, `\${${param.name}}`);
    });

    // Build request config
    const configParts = [];
    if (queryParams.length > 0) {
      const queryParamsStr = queryParams.map(p => `${p.name}`).join(', ');
      configParts.push(`params: { ${queryParamsStr} }`);
    }
    
    const config = configParts.length > 0 ? `, { ${configParts.join(', ')} }` : '';

    // Build method body
    const methodBody = bodyParam 
      ? `this.http.${method.httpMethod.toLowerCase()}(\`${path}\`, ${bodyParam.name}${config})`
      : `this.http.${method.httpMethod.toLowerCase()}(\`${path}\`${config})`;

    return `  async ${method.name}(${params}): Promise<${method.responseType}> {
    const response = await ${methodBody};
    return response.data;
  }`;
  }

  private generateMethodParams(method: ServiceMethod): string {
    if (method.parameters.length === 0) return '';
    
    const params = method.parameters.map(param => {
      const optional = param.required ? '' : '?';
      return `${param.name}${optional}: ${param.type}`;
    });
    
    return params.join(', ');
  }

  private async generateMainClient(services: ServiceDefinition[]): Promise<void> {
    const serviceImports = services.map(service => 
      `import { ${this.capitalize(service.name)}Client } from './${service.name}Client';`
    ).join('\n');

    const serviceProperties = services.map(service => 
      `  readonly ${service.name}: ${this.capitalize(service.name)}Client;`
    ).join('\n');

    const serviceInit = services.map(service => 
      `    this.${service.name} = new ${this.capitalize(service.name)}Client(this.http);`
    ).join('\n');

    const content = `import axios, { AxiosInstance, AxiosRequestConfig } from 'axios';
${serviceImports}

export interface RabbitMeshClientConfig {
  baseURL: string;
  timeout?: number;
  headers?: Record<string, string>;
}

export class RabbitMeshClient {
  private http: AxiosInstance;

${serviceProperties}

  constructor(config: RabbitMeshClientConfig | string) {
    const clientConfig = typeof config === 'string' 
      ? { baseURL: config }
      : config;

    this.http = axios.create({
      baseURL: clientConfig.baseURL,
      timeout: clientConfig.timeout || 10000,
      headers: {
        'Content-Type': 'application/json',
        ...clientConfig.headers
      }
    });

${serviceInit}
  }

  // Utility method to access underlying axios instance
  getHttpClient(): AxiosInstance {
    return this.http;
  }
}`;

    fs.writeFileSync(path.join(this.config.outputDir, 'client.ts'), content);
  }

  private async generatePackageJson(): Promise<void> {
    const packageJson = {
      name: this.config.packageName,
      version: '1.0.0',
      description: 'Auto-generated TypeScript client for RabbitMesh services',
      main: 'index.js',
      types: 'index.d.ts',
      scripts: {
        build: 'tsc'
      },
      dependencies: {
        axios: '^1.6.0'
      },
      devDependencies: {
        typescript: '^5.0.0',
        '@types/node': '^20.0.0'
      },
      keywords: ['rabbitmesh', 'microservices', 'api-client', 'typescript'],
      author: 'RabbitMesh Client Generator',
      license: 'MIT'
    };

    fs.writeFileSync(
      path.join(this.config.outputDir, 'package.json'), 
      JSON.stringify(packageJson, null, 2)
    );
  }

  private async generateTsConfig(): Promise<void> {
    const tsConfig = {
      compilerOptions: {
        target: 'ES2018',
        module: 'commonjs',
        outDir: './',
        rootDir: './',
        strict: true,
        esModuleInterop: true,
        skipLibCheck: true,
        forceConsistentCasingInFileNames: true,
        declaration: true
      },
      include: ['*.ts']
    };

    fs.writeFileSync(
      path.join(this.config.outputDir, 'tsconfig.json'), 
      JSON.stringify(tsConfig, null, 2)
    );
  }

  private async generateIndex(services: ServiceDefinition[]): Promise<void> {
    const exports = [
      "export { RabbitMeshClient } from './client';",
      "export * from './types';",
      ...services.map(service => `export { ${this.capitalize(service.name)}Client } from './${service.name}Client';`)
    ].join('\n');

    fs.writeFileSync(path.join(this.config.outputDir, 'index.ts'), exports + '\n');
  }

  private capitalize(str: string): string {
    return str.charAt(0).toUpperCase() + str.slice(1);
  }
}