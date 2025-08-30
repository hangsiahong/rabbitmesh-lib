import axios, { AxiosInstance } from 'axios';
import * as Types from './types';

export class AuthClient {
  constructor(private http: AxiosInstance) {}

  async login(): Promise<any> {
    const response = await this.http.get(`/api/v1/auth-service/auth/login`);
    return response.data;
  }

  async validateToken(): Promise<any> {
    const response = await this.http.get(`/api/v1/auth-service/auth/validate`);
    return response.data;
  }

  async refreshToken(): Promise<any> {
    const response = await this.http.get(`/api/v1/auth-service/auth/refresh`);
    return response.data;
  }

  async checkPermission(): Promise<any> {
    const response = await this.http.get(`/api/v1/auth-service/auth/check-permission`);
    return response.data;
  }

  async logout(): Promise<any> {
    const response = await this.http.get(`/api/v1/auth-service/auth/logout`);
    return response.data;
  }

  async getCurrentUser(): Promise<any> {
    const response = await this.http.get(`/api/v1/auth-service/auth/me`);
    return response.data;
  }
}