"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.AuthClient = void 0;
class AuthClient {
    constructor(http) {
        this.http = http;
    }
    async register(data) {
        const response = await this.http.post(`/api/v1/auth-service/register`, data);
        return response.data;
    }
    async login(data) {
        const response = await this.http.post(`/api/v1/auth-service/login`, data);
        return response.data;
    }
    async getProfile() {
        const response = await this.http.get(`/api/v1/auth-service/get_profile`);
        return response.data;
    }
    async listUsers() {
        const response = await this.http.get(`/api/v1/auth-service/list_users`);
        return response.data;
    }
}
exports.AuthClient = AuthClient;
