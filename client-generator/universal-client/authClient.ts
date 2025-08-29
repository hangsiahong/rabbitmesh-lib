import axios, { AxiosInstance } from 'axios';
import * as Types from './types';

export class AuthClient {
  constructor(private http: AxiosInstance) {}

  async register(data: any): Promise<any> {
    const response = await this.http.post(`/api/v1/auth-service/register`, data);
    return response.data;
  }

  async login(data: any): Promise<any> {
    const response = await this.http.post(`/api/v1/auth-service/login`, data);
    return response.data;
  }

  async getProfile(): Promise<any> {
    const response = await this.http.get(`/api/v1/auth-service/get_profile`);
    return response.data;
  }

  async listUsers(): Promise<any> {
    const response = await this.http.get(`/api/v1/auth-service/list_users`);
    return response.data;
  }
}