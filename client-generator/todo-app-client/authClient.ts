import axios, { AxiosInstance } from 'axios';
import * as Types from './types';

export class AuthClient {
  constructor(private http: AxiosInstance) {}

  async register(username: string, email: string, password: string): Promise<any> {
    const response = await this.http.post(`/api/v1/auth-service/register`, { username, email, password });
    return response.data;
  }

  async login(username: string, password: string): Promise<any> {
    const response = await this.http.post(`/api/v1/auth-service/login`, { username, password });
    return response.data;
  }

  async getProfile(user_id: string): Promise<any> {
    const response = await this.http.get(`/api/v1/auth-service/get_profile`, { params: { user_id } });
    return response.data;
  }

  async listUsers(): Promise<any> {
    const response = await this.http.get(`/api/v1/auth-service/list_users`);
    return response.data;
  }
}