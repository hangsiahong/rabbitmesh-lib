import axios, { AxiosInstance, AxiosRequestConfig } from 'axios';
import { AuthClient } from './authClient';
import { BlogClient } from './blogClient';

export interface RabbitMeshClientConfig {
  baseURL: string;
  timeout?: number;
  headers?: Record<string, string>;
}

export class RabbitMeshClient {
  private http: AxiosInstance;

  readonly auth: AuthClient;
  readonly blog: BlogClient;

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

    this.auth = new AuthClient(this.http);
    this.blog = new BlogClient(this.http);
  }

  // Utility method to access underlying axios instance
  getHttpClient(): AxiosInstance {
    return this.http;
  }
}