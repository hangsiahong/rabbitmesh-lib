import axios, { AxiosInstance } from 'axios';
import * as Types from './types';

export class OrderClient {
  constructor(private http: AxiosInstance) {}

  async createOrder(): Promise<any> {
    const response = await this.http.get(`/api/v1/order-service/orders`);
    return response.data;
  }

  async getOrder(): Promise<any> {
    const response = await this.http.get(`/api/v1/order-service/orders/{id}`);
    return response.data;
  }

  async updateOrder(): Promise<any> {
    const response = await this.http.get(`/api/v1/order-service/orders/{id}`);
    return response.data;
  }

  async deleteOrder(): Promise<any> {
    const response = await this.http.get(`/api/v1/order-service/orders/{id}`);
    return response.data;
  }

  async getUserOrders(): Promise<any> {
    const response = await this.http.get(`/api/v1/order-service/orders/user/{user_id}`);
    return response.data;
  }

  async getOrdersByStatus(): Promise<any> {
    const response = await this.http.get(`/api/v1/order-service/orders/status/{status}`);
    return response.data;
  }

  async confirmOrder(): Promise<any> {
    const response = await this.http.get(`/api/v1/order-service/orders/{id}/confirm`);
    return response.data;
  }

  async cancelOrder(): Promise<any> {
    const response = await this.http.get(`/api/v1/order-service/orders/{id}/cancel`);
    return response.data;
  }
}