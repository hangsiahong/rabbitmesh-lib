import axios, { AxiosInstance, AxiosRequestConfig } from 'axios';
import { AuthClient } from './authClient';
import { TodoClient } from './todoClient';
import { NotificationClient } from './notificationClient';

export interface RabbitMeshClientConfig {
  baseURL: string;
  timeout?: number;
  headers?: Record<string, string>;
}

export class RabbitMeshClient {
  private http: AxiosInstance;

  readonly auth: AuthClient;
  readonly todo: TodoClient;
  readonly notification: NotificationClient;

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
    this.todo = new TodoClient(this.http);
    this.notification = new NotificationClient(this.http);
  }

  // Utility method to access underlying axios instance
  getHttpClient(): AxiosInstance {
    return this.http;
  }
}