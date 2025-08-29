import { AxiosInstance } from 'axios';
import { CreateTodoRequest, UpdateTodoRequest, TodoResponse, TodoListResponse } from './types';
export declare class TodoClient {
    private http;
    constructor(http: AxiosInstance);
    createTodo(request: CreateTodoRequest): Promise<TodoResponse>;
    getTodo(id: string): Promise<TodoResponse>;
    listTodos(): Promise<TodoListResponse>;
    updateTodo(id: string, request: UpdateTodoRequest): Promise<TodoResponse>;
    deleteTodo(id: string): Promise<TodoResponse>;
    completeTodo(id: string): Promise<TodoResponse>;
    getStats(): Promise<any>;
}
//# sourceMappingURL=todoClient.d.ts.map