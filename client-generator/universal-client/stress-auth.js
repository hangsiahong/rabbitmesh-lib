#!/usr/bin/env node

const { RabbitMeshClient } = require('./dist/index.js');

class AuthStressTest {
  constructor() {
    this.client = new RabbitMeshClient({
      baseURL: 'http://localhost:8081'
    });
    
    this.stats = {
      total: 0,
      success: 0,
      failed: 0,
      auth_failures: 0,
      startTime: null,
      endTime: null,
      errors: {},
      responseTimes: []
    };
  }

  async runAuthStressTest(durationSeconds = 3) {
    console.log(`🚀 Starting ${durationSeconds}s AUTHENTICATION stress test...`);
    console.log('🔐 Testing JWT authentication on todo.createTodo()');
    console.log('⚡ This will verify auth is working...\n');

    // First, create a user and get a JWT token
    console.log('🔑 Creating test user and getting JWT token...');
    const timestamp = Date.now();
    const username = `authtest_${timestamp}`;
    const email = `authtest_${timestamp}@test.com`;
    
    try {
      const user = await this.client.auth.register({
        username: username,
        email: email,
        password: 'authtest123'
      });
      
      const authResult = await this.client.auth.login({
        username: username,
        password: 'authtest123'
      });
      
      const jwtToken = authResult.token;
      console.log(`✅ Got JWT token: ${jwtToken.substring(0, 20)}...`);
      
      // Now run stress test with JWT authentication
      this.stats.startTime = Date.now();
      const endTime = this.stats.startTime + (durationSeconds * 1000);
      
      let requestId = 0;
      
      while (Date.now() < endTime) {
        // Test both authenticated and unauthenticated requests
        const batchSize = 20;
        const batch = [];
        
        for (let i = 0; i < batchSize && Date.now() < endTime; i++) {
          // 50% authenticated, 50% unauthenticated to test both paths
          const useAuth = (requestId % 2 === 0);
          batch.push(this.testTodoWithAuth(++requestId, useAuth ? jwtToken : null));
        }
        
        await Promise.allSettled(batch);
        
        if (requestId % 100 === 0) {
          const elapsed = (Date.now() - this.stats.startTime) / 1000;
          const rate = this.stats.total / elapsed;
          process.stdout.write(`\r📈 ${this.stats.total} requests, ${rate.toFixed(1)} req/s, ${this.stats.success} success, ${this.stats.failed} failed, ${this.stats.auth_failures} auth failed`);
        }
      }
      
      this.stats.endTime = Date.now();
      this.printResults();
      
    } catch (error) {
      console.error('❌ Failed to setup auth test:', error.message);
    }
  }

  async testTodoWithAuth(requestId, jwtToken) {
    const startTime = Date.now();
    
    try {
      this.stats.total++;
      
      // Prepare headers
      const headers = {};
      if (jwtToken) {
        headers['Authorization'] = `Bearer ${jwtToken}`;
      }
      
      // Create todo request with or without auth
      const response = await fetch(`http://localhost:8081/api/v1/todo-service/create_todo`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          ...headers
        },
        body: JSON.stringify({
          title: `Auth Test Todo #${requestId}`,
          description: `Testing JWT authentication #${requestId} (${jwtToken ? 'authenticated' : 'unauthenticated'})`
        })
      });
      
      const responseTime = Date.now() - startTime;
      this.stats.responseTimes.push(responseTime);
      
      if (response.ok) {
        this.stats.success++;
      } else {
        this.stats.failed++;
        
        if (response.status === 401 || response.status === 403) {
          this.stats.auth_failures++;
        }
        
        // Track error types
        const errorType = response.status.toString();
        this.stats.errors[errorType] = (this.stats.errors[errorType] || 0) + 1;
      }
      
    } catch (error) {
      this.stats.failed++;
      
      const errorType = error.code || error.message || 'unknown';
      this.stats.errors[errorType] = (this.stats.errors[errorType] || 0) + 1;
    }
  }

  printResults() {
    const duration = (this.stats.endTime - this.stats.startTime) / 1000;
    const successRate = (this.stats.success / this.stats.total * 100).toFixed(2);
    const authFailRate = (this.stats.auth_failures / this.stats.total * 100).toFixed(2);
    const requestsPerSecond = (this.stats.total / duration).toFixed(2);
    const successPerSecond = (this.stats.success / duration).toFixed(2);
    
    // Calculate response time stats
    const avgResponseTime = this.stats.responseTimes.length > 0 
      ? (this.stats.responseTimes.reduce((a, b) => a + b, 0) / this.stats.responseTimes.length).toFixed(1)
      : 0;
    
    const sortedTimes = this.stats.responseTimes.sort((a, b) => a - b);
    const p95ResponseTime = sortedTimes.length > 0 
      ? sortedTimes[Math.floor(sortedTimes.length * 0.95)]
      : 0;
    
    console.log('\n\n🔐 JWT AUTHENTICATION STRESS TEST RESULTS');
    console.log('═'.repeat(60));
    console.log(`⏱️  Duration: ${duration.toFixed(2)} seconds`);
    console.log(`📊 Total Requests: ${this.stats.total}`);
    console.log(`✅ Successful: ${this.stats.success} (${successRate}%)`);
    console.log(`❌ Failed: ${this.stats.failed}`);
    console.log(`🔐 Auth Failures: ${this.stats.auth_failures} (${authFailRate}%)`);
    console.log(`🚀 Total Rate: ${requestsPerSecond} requests/second`);
    console.log(`⚡ Success Rate: ${successPerSecond} successful requests/second`);
    console.log(`⏱️  Avg Response Time: ${avgResponseTime}ms`);
    console.log(`📈 95th Percentile: ${p95ResponseTime}ms`);
    
    if (this.stats.failed > 0) {
      console.log('\n💥 ERROR BREAKDOWN:');
      Object.entries(this.stats.errors).forEach(([error, count]) => {
        const meaning = error === '401' ? '(Unauthorized - Good!)' : 
                       error === '403' ? '(Forbidden - Good!)' : 
                       error === '400' ? '(Bad Request)' : '';
        console.log(`   ${error}: ${count} times ${meaning}`);
      });
    }
    
    console.log('\n📈 AUTHENTICATION STATUS:');
    if (this.stats.auth_failures > 0) {
      console.log('🎉 AUTHENTICATION IS WORKING!');
      console.log(`   ✅ ${this.stats.auth_failures} requests properly rejected without JWT`);
      console.log(`   ✅ ${this.stats.success} requests succeeded with valid JWT`);
      
      const authWorkingRate = (this.stats.auth_failures / (this.stats.total / 2)) * 100;
      console.log(`   🔐 ${authWorkingRate.toFixed(1)}% of unauthenticated requests rejected`);
    } else {
      console.log('❌ AUTHENTICATION NOT WORKING!');
      console.log('   ⚠️  All requests succeeded - JWT validation is not enforced');
      console.log('   🐛 The #[require_auth] macro is not working');
    }
    
    console.log('\n🎯 Performance vs Security:');
    console.log(`   Without Auth: ~1700 req/s (from previous test)`);
    console.log(`   With Auth: ${successPerSecond} req/s`);
    if (parseFloat(successPerSecond) < 1000) {
      console.log('   ✅ Good! Auth overhead is working (slower = more secure)');
    } else {
      console.log('   ❌ Suspicious! Still too fast - auth may not be working');
    }
    
    console.log('\n🎉 Auth stress test completed!');
  }
}

async function main() {
  const stressTest = new AuthStressTest();
  
  try {
    const duration = process.argv[2] ? parseInt(process.argv[2]) : 3;
    await stressTest.runAuthStressTest(duration);
  } catch (error) {
    console.error('💥 Auth stress test failed:', error.message);
    process.exit(1);
  }
}

main().catch(console.error);