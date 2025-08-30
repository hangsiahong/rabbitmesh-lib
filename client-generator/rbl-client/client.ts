// Re-export all service hooks
export * from './authClient';
export * from './orderClient';
export * from './userClient';

// Axios configuration for the generated hooks
import axios from 'axios';

export interface RabbitMeshClientConfig {
  baseURL: string;
  timeout?: number;
  headers?: Record<string, string>;
}

// Configure axios defaults for all generated hooks
export function configureRabbitMeshClient(config: RabbitMeshClientConfig | string) {
  const clientConfig = typeof config === 'string' 
    ? { baseURL: config }
    : config;

  axios.defaults.baseURL = clientConfig.baseURL;
  axios.defaults.timeout = clientConfig.timeout || 10000;
  axios.defaults.headers.common['Content-Type'] = 'application/json';
  
  if (clientConfig.headers) {
    Object.assign(axios.defaults.headers.common, clientConfig.headers);
  }
}

// Example usage:
// import { configureRabbitMeshClient, useLogin, useCreateOrder } from '@your-org/rabbitmesh-client';
// 
// // Configure once in your app initialization
// configureRabbitMeshClient('http://localhost:3333');
// 
// // Use React Query hooks in your components
// const loginMutation = useLogin();
// const { data: orders } = useGetUserOrders({ user_id: '123' });