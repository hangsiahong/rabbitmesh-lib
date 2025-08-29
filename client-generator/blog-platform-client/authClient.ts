import axios, { AxiosInstance } from 'axios';
import * as Types from './types';

export class AuthClient {
  constructor(private http: AxiosInstance) {}

  async register(request: RegisterRequest): Promise<AuthResponse> {
    const response = await this.http.post(`/auth/register`, request);
    return response.data;
  }

  async login(request: LoginRequest): Promise<AuthResponse> {
    const response = await this.http.post(`/auth/login`, request);
    return response.data;
  }

  async validateToken(request: ValidateTokenRequest): Promise<ValidateTokenResponse> {
    const response = await this.http.post(`/auth/validate`, request);
    return response.data;
  }

  async getProfile(user_id: string): Promise<AuthResponse> {
    const response = await this.http.get(`/auth/profile/${user_id}`);
    return response.data;
  }

  async updateProfile(user_id: string, request: UpdateProfileRequest): Promise<AuthResponse> {
    const response = await this.http.put(`/auth/profile/${user_id}`, request);
    return response.data;
  }
}