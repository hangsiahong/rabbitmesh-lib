import axios, { AxiosInstance } from 'axios';
import { CreateTodoRequest, UpdateTodoRequest, TodoResponse, TodoListResponse } from './types';

export class TodoClient {
  constructor(private http: AxiosInstance) {}

  async createTodo(request: CreateTodoRequest): Promise<TodoResponse> {
    const response = await this.http.post(`/todos`, request);
    return response.data;
  }

  async getTodo(id: string): Promise<TodoResponse> {
    const response = await this.http.get(`/todos/${id}`);
    return response.data;
  }

  async listTodos(): Promise<TodoListResponse> {
    const response = await this.http.get(`/todos`);
    return response.data;
  }

  async updateTodo(id: string, request: UpdateTodoRequest): Promise<TodoResponse> {
    const response = await this.http.put(`/todos/${id}`, request);
    return response.data;
  }

  async deleteTodo(id: string): Promise<TodoResponse> {
    const response = await this.http.delete(`/todos/${id}`);
    return response.data;
  }

  async completeTodo(id: string): Promise<TodoResponse> {
    const response = await this.http.post(`/todos/${id}/complete`);
    return response.data;
  }

  async getStats(): Promise<any> {
    const response = await this.http.get(`/todos/stats`);
    return response.data;
  }
}