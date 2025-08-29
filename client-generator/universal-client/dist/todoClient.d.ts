import { AxiosInstance } from 'axios';
export declare class TodoClient {
    private http;
    constructor(http: AxiosInstance);
    createTodo(data: any): Promise<any>;
    getTodos(): Promise<any>;
    getTodo(): Promise<any>;
    updateTodo(data: any): Promise<any>;
    deleteTodo(): Promise<any>;
    getTodoStats(): Promise<any>;
}
