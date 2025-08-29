const { RabbitMeshClient } = require('./dist/index.js');

async function testClient() {
  console.log('🚀 Testing RabbitMesh Todo App Client...');
  
  // Create client instance pointing to our auto-gateway
  const client = new RabbitMeshClient('http://localhost:8080');
  
  try {
    console.log('\n📝 Testing user registration...');
    
    // Fix the auth client methods to pass proper request bodies
    const registerResponse = await client.auth.register('testclient', 'testclient@example.com', 'password123');
    console.log('✅ Registration successful:', registerResponse);
    
  } catch (error) {
    console.log('❌ Registration error:', error.response?.data || error.message);
  }
  
  try {
    console.log('\n📋 Testing todo creation...');
    const todoResponse = await client.todo.createTodo('Test todo from client', 'Created using generated TypeScript client', 'high');
    console.log('✅ Todo created:', todoResponse);
    
  } catch (error) {
    console.log('❌ Todo creation error:', error.response?.data || error.message);
  }
  
  try {
    console.log('\n🔔 Testing notification sending...');
    const notificationResponse = await client.notification.sendNotification('user123', 'email', 'Test Notification', 'This is a test notification from the generated client');
    console.log('✅ Notification sent:', notificationResponse);
    
  } catch (error) {
    console.log('❌ Notification error:', error.response?.data || error.message);
  }
  
  console.log('\n🎉 Client testing completed!');
}

testClient().catch(console.error);