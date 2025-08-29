import axios, { AxiosInstance } from 'axios';
import * as Types from './types';

export class TodoClient {
  constructor(private http: AxiosInstance) {}

  async createTodo(title: string, description?: string, priority?: string): Promise<any> {
    const response = await this.http.post(`/api/v1/todo-service/create_todo`, { title, description, priority });
    return response.data;
  }

  async getTodos(): Promise<any> {
    const response = await this.http.get(`/api/v1/todo-service/get_todos`);
    return response.data;
  }

  async getTodo(todo_id: string): Promise<any> {
    const response = await this.http.get(`/api/v1/todo-service/get_todo`, { params: { todo_id } });
    return response.data;
  }

  async updateTodo(todo_id: string, title?: string, description?: string, completed?: boolean): Promise<any> {
    const response = await this.http.put(`/api/v1/todo-service/update_todo`, { todo_id, title, description, completed });
    return response.data;
  }

  async deleteTodo(todo_id: string): Promise<any> {
    const response = await this.http.delete(`/api/v1/todo-service/delete_todo`, { params: { todo_id } });
    return response.data;
  }

  async getTodoStats(): Promise<any> {
    const response = await this.http.get(`/api/v1/todo-service/get_todo_stats`);
    return response.data;
  }
}