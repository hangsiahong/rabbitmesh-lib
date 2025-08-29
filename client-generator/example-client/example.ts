// Example usage of the generated RabbitMesh client
import { RabbitMeshClient } from './index';

async function runExample() {
  // Initialize client - usually you'd get this from environment variables
  const client = new RabbitMeshClient('http://localhost:3000');

  try {
    console.log('🚀 RabbitMesh Client Example\n');

    // Create a new todo - notice full autocomplete support!
    console.log('📝 Creating a new todo...');
    const newTodo = await client.todo.createTodo({
      title: "Learn RabbitMesh",
      description: "Understand the microservice architecture"
    });
    console.log('✅ Created:', newTodo);

    if (newTodo.success && newTodo.todo) {
      const todoId = newTodo.todo.id;

      // Get the todo we just created
      console.log('\n🔍 Fetching the todo...');
      const fetchedTodo = await client.todo.getTodo(todoId);
      console.log('✅ Fetched:', fetchedTodo);

      // Update the todo
      console.log('\n✏️ Updating the todo...');
      const updatedTodo = await client.todo.updateTodo(todoId, {
        description: "Updated description!",
        completed: false
      });
      console.log('✅ Updated:', updatedTodo);

      // Mark as completed
      console.log('\n✅ Marking todo as completed...');
      const completedTodo = await client.todo.completeTodo(todoId);
      console.log('✅ Completed:', completedTodo);
    }

    // List all todos
    console.log('\n📋 Listing all todos...');
    const allTodos = await client.todo.listTodos();
    console.log(`✅ Found ${allTodos.total} todos:`, allTodos.todos);

    // Get statistics
    console.log('\n📊 Getting statistics...');
    const stats = await client.todo.getStats();
    console.log('✅ Stats:', stats);

  } catch (error) {
    console.error('❌ Error:', error);
  }
}

// Run the example
if (require.main === module) {
  runExample().catch(console.error);
}

export { runExample };