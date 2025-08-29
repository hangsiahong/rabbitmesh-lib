"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.TodoClient = void 0;
class TodoClient {
    constructor(http) {
        this.http = http;
    }
    async createTodo(request) {
        const response = await this.http.post(`/todos`, request);
        return response.data;
    }
    async getTodo(id) {
        const response = await this.http.get(`/todos/${id}`);
        return response.data;
    }
    async listTodos() {
        const response = await this.http.get(`/todos`);
        return response.data;
    }
    async updateTodo(id, request) {
        const response = await this.http.put(`/todos/${id}`, request);
        return response.data;
    }
    async deleteTodo(id) {
        const response = await this.http.delete(`/todos/${id}`);
        return response.data;
    }
    async completeTodo(id) {
        const response = await this.http.post(`/todos/${id}/complete`);
        return response.data;
    }
    async getStats() {
        const response = await this.http.get(`/todos/stats`);
        return response.data;
    }
}
exports.TodoClient = TodoClient;
//# sourceMappingURL=todoClient.js.map