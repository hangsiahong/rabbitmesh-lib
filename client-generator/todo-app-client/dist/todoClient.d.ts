import { AxiosInstance } from 'axios';
export declare class TodoClient {
    private http;
    constructor(http: AxiosInstance);
    createTodo(title: string, description?: string, priority?: string): Promise<any>;
    getTodos(): Promise<any>;
    getTodo(todo_id: string): Promise<any>;
    updateTodo(todo_id: string, title?: string, description?: string, completed?: boolean): Promise<any>;
    deleteTodo(todo_id: string): Promise<any>;
    getTodoStats(): Promise<any>;
}
//# sourceMappingURL=todoClient.d.ts.map