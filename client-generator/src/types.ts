// Type definitions for service parsing and client generation

export interface ServiceDefinition {
  name: string;
  methods: ServiceMethod[];
  version?: string;
  description?: string;
}

export interface ServiceMethod {
  name: string;
  httpMethod: string;
  path: string;
  requestType?: string;
  responseType: string;
  parameters: Parameter[];
  description?: string;
}

export interface Parameter {
  name: string;
  type: string;
  required: boolean;
  source: 'path' | 'query' | 'body';
  description?: string;
}

export interface TypeDefinition {
  name: string;
  fields: TypeField[];
  description?: string;
}

export interface TypeField {
  name: string;
  type: string;
  optional: boolean;
  description?: string;
}

export interface GeneratorConfig {
  gatewayUrl: string;
  outputDir: string;
  packageName: string;
  includeServices?: string[];
  excludeServices?: string[];
}