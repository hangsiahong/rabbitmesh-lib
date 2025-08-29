# RabbitMesh TypeScript Client

Auto-generated TypeScript client for RabbitMesh microservices with full type safety and autocomplete support.

## Installation

```bash
npm install @rabbitmesh/todo-client
```

## Quick Start

```typescript
import { RabbitMeshClient } from '@rabbitmesh/todo-client';

// Initialize the client
const client = new RabbitMeshClient('http://localhost:3000');

// Create a todo with full autocomplete! ✨
const newTodo = await client.todo.createTodo({
  title: "Learn TypeScript",
  description: "Master the language"
});

console.log(newTodo); // Full type safety
```

## Usage Examples

### React Hook Example

```typescript
// useTodos.ts
import { useState, useEffect } from 'react';
import { RabbitMeshClient, Todo } from '@rabbitmesh/todo-client';

const client = new RabbitMeshClient(process.env.REACT_APP_API_URL!);

export function useTodos() {
  const [todos, setTodos] = useState<Todo[]>([]);
  const [loading, setLoading] = useState(false);

  const fetchTodos = async () => {
    setLoading(true);
    try {
      const response = await client.todo.listTodos();
      setTodos(response.todos);
    } catch (error) {
      console.error('Failed to fetch todos:', error);
    } finally {
      setLoading(false);
    }
  };

  const createTodo = async (title: string, description?: string) => {
    const response = await client.todo.createTodo({ title, description });
    if (response.success) {
      await fetchTodos(); // Refresh list
    }
    return response;
  };

  const updateTodo = async (id: string, updates: { title?: string; description?: string; completed?: boolean }) => {
    const response = await client.todo.updateTodo(id, updates);
    if (response.success) {
      await fetchTodos(); // Refresh list
    }
    return response;
  };

  useEffect(() => {
    fetchTodos();
  }, []);

  return {
    todos,
    loading,
    createTodo,
    updateTodo,
    fetchTodos
  };
}
```

### React Component Example

```tsx
// TodoList.tsx
import React from 'react';
import { useTodos } from './useTodos';

export function TodoList() {
  const { todos, loading, createTodo, updateTodo } = useTodos();

  const handleCreateTodo = async () => {
    await createTodo("New Todo", "Description here");
  };

  const toggleComplete = async (todo: Todo) => {
    await updateTodo(todo.id, { completed: !todo.completed });
  };

  if (loading) return <div>Loading...</div>;

  return (
    <div>
      <button onClick={handleCreateTodo}>Add Todo</button>
      {todos.map(todo => (
        <div key={todo.id}>
          <h3>{todo.title}</h3>
          <p>{todo.description}</p>
          <label>
            <input
              type="checkbox"
              checked={todo.completed}
              onChange={() => toggleComplete(todo)}
            />
            Completed
          </label>
        </div>
      ))}
    </div>
  );
}
```

### Next.js API Route Example

```typescript
// pages/api/todos/[id].ts
import { NextApiRequest, NextApiResponse } from 'next';
import { RabbitMeshClient } from '@rabbitmesh/todo-client';

const client = new RabbitMeshClient(process.env.BACKEND_URL!);

export default async function handler(req: NextApiRequest, res: NextApiResponse) {
  const { id } = req.query;

  try {
    if (req.method === 'GET') {
      const todo = await client.todo.getTodo(id as string);
      res.status(200).json(todo);
    } else if (req.method === 'PUT') {
      const updated = await client.todo.updateTodo(id as string, req.body);
      res.status(200).json(updated);
    } else if (req.method === 'DELETE') {
      const deleted = await client.todo.deleteTodo(id as string);
      res.status(200).json(deleted);
    }
  } catch (error) {
    res.status(500).json({ error: 'Failed to process request' });
  }
}
```

### Node.js Server Example

```typescript
// server.ts
import { RabbitMeshClient } from '@rabbitmesh/todo-client';

const client = new RabbitMeshClient({
  baseURL: 'http://localhost:3000',
  timeout: 5000,
  headers: {
    'Authorization': 'Bearer your-token-here'
  }
});

async function main() {
  try {
    // Create a new todo
    const newTodo = await client.todo.createTodo({
      title: "Server-side todo",
      description: "Created from Node.js"
    });
    
    console.log('Created:', newTodo);

    // Get statistics
    const stats = await client.todo.getStats();
    console.log('Stats:', stats);

    // List all todos
    const allTodos = await client.todo.listTodos();
    console.log(`Total todos: ${allTodos.total}`);

  } catch (error) {
    console.error('Error:', error);
  }
}

main();
```

## Available Methods

All methods return Promises and include full TypeScript types:

### Todo Service

- `client.todo.createTodo(request: CreateTodoRequest): Promise<TodoResponse>`
- `client.todo.getTodo(id: string): Promise<TodoResponse>`
- `client.todo.listTodos(): Promise<TodoListResponse>`
- `client.todo.updateTodo(id: string, request: UpdateTodoRequest): Promise<TodoResponse>`
- `client.todo.deleteTodo(id: string): Promise<TodoResponse>`
- `client.todo.completeTodo(id: string): Promise<TodoResponse>`
- `client.todo.getStats(): Promise<any>`

## Type Definitions

All request/response types are fully typed:

```typescript
interface Todo {
  id: string;
  title: string;
  description?: string | null;
  completed: boolean;
  created_at: string;
  updated_at: string;
}

interface CreateTodoRequest {
  title: string;
  description?: string;
}

interface UpdateTodoRequest {
  title?: string;
  description?: string;
  completed?: boolean;
}

// ... and more
```

## Configuration

```typescript
const client = new RabbitMeshClient({
  baseURL: 'http://localhost:3000',
  timeout: 10000, // 10 seconds
  headers: {
    'Authorization': 'Bearer token',
    'Custom-Header': 'value'
  }
});
```

## Error Handling

```typescript
try {
  const todo = await client.todo.getTodo('invalid-id');
} catch (error) {
  if (error.response?.status === 404) {
    console.log('Todo not found');
  } else {
    console.error('Unexpected error:', error);
  }
}
```

## Development

```bash
# Install dependencies
npm install

# Build the client
npm run build

# The built files will be in ./dist/
```