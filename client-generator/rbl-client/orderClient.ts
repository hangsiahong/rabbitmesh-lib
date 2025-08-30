import axios, { AxiosInstance } from 'axios';
import * as Types from './types';

export class OrderClient {
  constructor(private http: AxiosInstance) {}

  async createOrder(data: any): Promise<any> {
    const response = await this.http.post(`/api/v1/order-service/orders`, data);
    return response.data;
  }

  async getOrder(): Promise<any> {
    const response = await this.http.get(`/api/v1/order-service/orders/{id}`);
    return response.data;
  }

  async updateOrder(data: any): Promise<any> {
    const response = await this.http.put(`/api/v1/order-service/orders/{id}`, data);
    return response.data;
  }

  async deleteOrder(): Promise<any> {
    const response = await this.http.delete(`/api/v1/order-service/orders/{id}`);
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

  async confirmOrder(data: any): Promise<any> {
    const response = await this.http.post(`/api/v1/order-service/orders/{id}/confirm`, data);
    return response.data;
  }

  async cancelOrder(data: any): Promise<any> {
    const response = await this.http.post(`/api/v1/order-service/orders/{id}/cancel`, data);
    return response.data;
  }
}