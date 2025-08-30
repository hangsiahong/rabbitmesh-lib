import axios, { AxiosInstance } from 'axios';
import * as Types from './types';

export class UserClient {
  constructor(private http: AxiosInstance) {}

  async createUser(): Promise<any> {
    const response = await this.http.get(`/api/v1/user-service/users`);
    return response.data;
  }

  async getUser(): Promise<any> {
    const response = await this.http.get(`/api/v1/user-service/users/{id}`);
    return response.data;
  }

  async updateUser(): Promise<any> {
    const response = await this.http.get(`/api/v1/user-service/users/{id}`);
    return response.data;
  }

  async deleteUser(): Promise<any> {
    const response = await this.http.get(`/api/v1/user-service/users/{id}`);
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