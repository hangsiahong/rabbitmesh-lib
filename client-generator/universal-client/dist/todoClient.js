"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.TodoClient = void 0;
class TodoClient {
    constructor(http) {
        this.http = http;
    }
    async createTodo(data) {
        const response = await this.http.post(`/api/v1/todo-service/create_todo`, data);
        return response.data;
    }
    async getTodos() {
        const response = await this.http.get(`/api/v1/todo-service/get_todos`);
        return response.data;
    }
    async getTodo() {
        const response = await this.http.get(`/api/v1/todo-service/get_todo`);
        return response.data;
    }
    async updateTodo(data) {
        const response = await this.http.put(`/api/v1/todo-service/update_todo`, data);
        return response.data;
    }
    async deleteTodo() {
        const response = await this.http.delete(`/api/v1/todo-service/delete_todo`);
        return response.data;
    }
    async getTodoStats() {
        const response = await this.http.get(`/api/v1/todo-service/get_todo_stats`);
        return response.data;
    }
}
exports.TodoClient = TodoClient;
