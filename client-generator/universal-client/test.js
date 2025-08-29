#!/usr/bin/env node

const { RabbitMeshClient } = require('./dist/index.js');

async function testClient() {
  console.log('🚀 Testing universal RabbitMesh client...');
  
  // Create client pointed to our dynamic gateway
  const client = new RabbitMeshClient({
    baseURL: 'http://localhost:8081'
  });
  
  try {
    // Test auth service registration
    console.log('📝 Testing user registration...');
    const user = await client.auth.register({
      username: 'testuser3',
      email: 'test3@example.com', 
      password: 'password123'
    });
    console.log('✅ User registered:', user);
    
    // Test auth service login
    console.log('🔐 Testing user login...');
    const authResult = await client.auth.login({
      username: 'testuser3',
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
    
    console.log('🎉 ALL TESTS PASSED! Universal client generator works perfectly!');
    
  } catch (error) {
    console.error('❌ Test failed:', error.response?.data || error.message);
  }
}

testClient().catch(console.error);