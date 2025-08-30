#!/usr/bin/env node

const { RabbitMeshClient } = require('./dist/index.js');

async function testClient() {
  console.log('🚀 Testing universal RabbitMesh client...');
  
  // Create client pointed to our dynamic gateway
  const client = new RabbitMeshClient({
    baseURL: 'http://localhost:8081'
  });
  
  try {
    // Generate unique username to avoid conflicts
    const timestamp = Date.now();
    const username = `testuser_${timestamp}`;
    const email = `test_${timestamp}@example.com`;
    
    // Test auth service registration
    console.log('📝 Testing user registration...');
    const user = await client.auth.register({
      username: username,
      email: email,
      password: 'password123'
    });
    console.log('✅ User registered:', user);
    
    // Test auth service login
    console.log('🔐 Testing user login...');
    const authResult = await client.auth.login({
      username: username,
      password: 'password123'
    });
    console.log('✅ User logged in:', authResult);
    
    // Test todo service
    console.log('📋 Testing create todo...');
    const todo = await client.todo.createTodo({
      title: 'Test Todo from Universal Client',
      description: 'This todo was created using the UNIVERSAL client generator that works with ANY project!'
    });
    console.log('✅ Todo created:', todo);
    
    // Test notification service
    console.log('🔔 Testing send notification...');
    const notification = await client.notification.sendNotification({
      user_id: user.user_id,
      title: 'Test Notification',
      message: 'Test notification from Universal Client',
      recipient: email,
      notification_type: 'WelcomeMessage',
      metadata: {}
    });
    console.log('✅ Notification sent:', notification);
    
    console.log('🎉 ALL TESTS PASSED! Universal client generator works perfectly!');
    
  } catch (error) {
    console.error('❌ Test failed:', error.response?.data || error.message);
  }
}

testClient().catch(console.error);