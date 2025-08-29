export interface Todo {
    id: string;
    title: string;
    description?: string | null;
    completed: boolean;
    created_at: string;
    updated_at: string;
}
export interface CreateTodoRequest {
    title: string;
    description?: string;
}
export interface UpdateTodoRequest {
    title?: string;
    description?: string;
    completed?: boolean;
}
export interface TodoResponse {
    success: boolean;
    message: string;
    todo?: Todo | null;
}
export interface TodoListResponse {
    success: boolean;
    message: string;
    todos: Todo[];
    total: number;
}
//# sourceMappingURL=types.d.ts.map