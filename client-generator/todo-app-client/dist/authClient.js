"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.AuthClient = void 0;
class AuthClient {
    constructor(http) {
        this.http = http;
    }
    async register(username, email, password) {
        const response = await this.http.post(`/api/v1/auth-service/register`, { username, email, password });
        return response.data;
    }
    async login(username, password) {
        const response = await this.http.post(`/api/v1/auth-service/login`, { username, password });
        return response.data;
    }
    async getProfile(user_id) {
        const response = await this.http.get(`/api/v1/auth-service/get_profile`, { params: { user_id } });
        return response.data;
    }
    async listUsers() {
        const response = await this.http.get(`/api/v1/auth-service/list_users`);
        return response.data;
    }
}
exports.AuthClient = AuthClient;
