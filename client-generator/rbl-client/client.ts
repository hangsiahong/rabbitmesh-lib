import axios, { AxiosInstance, AxiosRequestConfig } from 'axios';
import { AuthClient } from './authClient';
import { OrderClient } from './orderClient';
import { UserClient } from './userClient';

export interface RabbitMeshClientConfig {
  baseURL: string;
  timeout?: number;
  headers?: Record<string, string>;
}

export class RabbitMeshClient {
  private http: AxiosInstance;

  readonly auth: AuthClient;
  readonly order: OrderClient;
  readonly user: UserClient;

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
    this.order = new OrderClient(this.http);
    this.user = new UserClient(this.http);
  }

  // Utility method to access underlying axios instance
  getHttpClient(): AxiosInstance {
    return this.http;
  }
}