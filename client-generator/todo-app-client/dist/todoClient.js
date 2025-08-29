"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.TodoClient = void 0;
class TodoClient {
    constructor(http) {
        this.http = http;
    }
    async createTodo(title, description, priority) {
        const response = await this.http.post(`/api/v1/todo-service/create_todo`, { title, description, priority });
        return response.data;
    }
    async getTodos() {
        const response = await this.http.get(`/api/v1/todo-service/get_todos`);
        return response.data;
    }
    async getTodo(todo_id) {
        const response = await this.http.get(`/api/v1/todo-service/get_todo`, { params: { todo_id } });
        return response.data;
    }
    async updateTodo(todo_id, title, description, completed) {
        const response = await this.http.put(`/api/v1/todo-service/update_todo`, { todo_id, title, description, completed });
        return response.data;
    }
    async deleteTodo(todo_id) {
        const response = await this.http.delete(`/api/v1/todo-service/delete_todo`, { params: { todo_id } });
        return response.data;
    }
    async getTodoStats() {
        const response = await this.http.get(`/api/v1/todo-service/get_todo_stats`);
        return response.data;
    }
}
exports.TodoClient = TodoClient;
