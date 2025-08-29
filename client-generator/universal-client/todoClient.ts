import axios, { AxiosInstance } from 'axios';
import * as Types from './types';

export class TodoClient {
  constructor(private http: AxiosInstance) {}

  async createTodo(data: any): Promise<any> {
    const response = await this.http.post(`/api/v1/todo-service/create_todo`, data);
    return response.data;
  }

  async getTodos(): Promise<any> {
    const response = await this.http.get(`/api/v1/todo-service/get_todos`);
    return response.data;
  }

  async getTodo(): Promise<any> {
    const response = await this.http.get(`/api/v1/todo-service/get_todo`);
    return response.data;
  }

  async updateTodo(data: any): Promise<any> {
    const response = await this.http.put(`/api/v1/todo-service/update_todo`, data);
    return response.data;
  }

  async deleteTodo(): Promise<any> {
    const response = await this.http.delete(`/api/v1/todo-service/delete_todo`);
    return response.data;
  }

  async getTodoStats(): Promise<any> {
    const response = await this.http.get(`/api/v1/todo-service/get_todo_stats`);
    return response.data;
  }
}