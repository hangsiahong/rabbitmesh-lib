import axios, { AxiosInstance } from 'axios';
import * as Types from './types';

export class AuthClient {
  constructor(private http: AxiosInstance) {}

  async login(data: any): Promise<any> {
    const response = await this.http.post(`/api/v1/auth-service/auth/login`, data);
    return response.data;
  }

  async validateToken(data: any): Promise<any> {
    const response = await this.http.post(`/api/v1/auth-service/auth/validate`, data);
    return response.data;
  }

  async refreshToken(data: any): Promise<any> {
    const response = await this.http.post(`/api/v1/auth-service/auth/refresh`, data);
    return response.data;
  }

  async checkPermission(data: any): Promise<any> {
    const response = await this.http.post(`/api/v1/auth-service/auth/check-permission`, data);
    return response.data;
  }

  async logout(data: any): Promise<any> {
    const response = await this.http.post(`/api/v1/auth-service/auth/logout`, data);
    return response.data;
  }

  async getCurrentUser(): Promise<any> {
    const response = await this.http.get(`/api/v1/auth-service/auth/me`);
    return response.data;
  }
}