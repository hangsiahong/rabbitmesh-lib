import axios, { AxiosInstance } from 'axios';
import * as Types from './types';

export class UserClient {
  constructor(private http: AxiosInstance) {}

  async createUser(data: any): Promise<any> {
    const response = await this.http.post(`/api/v1/user-service/users`, data);
    return response.data;
  }

  async getUser(): Promise<any> {
    const response = await this.http.get(`/api/v1/user-service/users/{id}`);
    return response.data;
  }

  async updateUser(data: any): Promise<any> {
    const response = await this.http.put(`/api/v1/user-service/users/{id}`, data);
    return response.data;
  }

  async deleteUser(): Promise<any> {
    const response = await this.http.delete(`/api/v1/user-service/users/{id}`);
    return response.data;
  }

  async listUsers(): Promise<any> {
    const response = await this.http.get(`/api/v1/user-service/users`);
    return response.data;
  }

  async getUserByEmail(): Promise<any> {
    const response = await this.http.get(`/api/v1/user-service/users/email/{email}`);
    return response.data;
  }
}