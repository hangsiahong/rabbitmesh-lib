#!/usr/bin/env node

const { RabbitMeshClient } = require('./dist/index.js');

class StressTest {
  constructor() {
    this.client = new RabbitMeshClient({
      baseURL: 'http://localhost:8081'
    });
    
    this.stats = {
      total: 0,
      success: 0,
      failed: 0,
      startTime: null,
      endTime: null,
      errors: {}
    };
  }

  async runStressTest(durationSeconds = 30) {
    console.log(`🚀 Starting ${durationSeconds}s stress test on RabbitMesh framework...`);
    console.log('📊 Testing all services: auth, todo, notification');
    console.log('⚡ Sending burst requests as fast as possible...\n');

    this.stats.startTime = Date.now();
    const endTime = this.stats.startTime + (durationSeconds * 1000);
    
    // Run concurrent requests until time expires
    const promises = [];
    let requestId = 0;
    
    while (Date.now() < endTime) {
      // Create batches of concurrent requests
      const batchSize = 10; // Send 10 requests concurrently
      const batch = [];
      
      for (let i = 0; i < batchSize && Date.now() < endTime; i++) {
        batch.push(this.performSingleTest(++requestId));
      }
      
      // Wait for current batch to complete before starting next
      await Promise.allSettled(batch);
      
      // Quick progress update every 100 requests
      if (requestId % 100 === 0) {
        const elapsed = (Date.now() - this.stats.startTime) / 1000;
        const rate = this.stats.total / elapsed;
        process.stdout.write(`\r📈 ${this.stats.total} requests, ${rate.toFixed(1)} req/s, ${this.stats.success} success, ${this.stats.failed} failed`);
      }
    }
    
    this.stats.endTime = Date.now();
    this.printResults();
  }

  async performSingleTest(requestId) {
    const timestamp = Date.now() + requestId; // Unique timestamp per request
    const username = `stress_${timestamp}`;
    const email = `stress_${timestamp}@test.com`;
    
    try {
      this.stats.total++;
      
      // Test 1: Register user
      await this.client.auth.register({
        username: username,
        email: email,
        password: 'stress123'
      });
      
      // Test 2: Login user  
      const authResult = await this.client.auth.login({
        username: username,
        password: 'stress123'
      });
      
      // Test 3: Create todo
      await this.client.todo.createTodo({
        title: `Stress Todo ${requestId}`,
        description: `Created during stress test #${requestId}`
      });
      
      // Test 4: Send notification
      await this.client.notification.sendNotification({
        user_id: authResult.user.id,
        title: `Stress Notification ${requestId}`,
        message: `Stress test notification #${requestId}`,
        recipient: email,
        notification_type: 'SystemAlert',
        metadata: { test_id: requestId }
      });
      
      this.stats.success++;
      
    } catch (error) {
      this.stats.failed++;
      
      // Track error types
      const errorType = error.response?.status || error.code || 'unknown';
      this.stats.errors[errorType] = (this.stats.errors[errorType] || 0) + 1;
    }
  }

  printResults() {
    const duration = (this.stats.endTime - this.stats.startTime) / 1000;
    const successRate = (this.stats.success / this.stats.total * 100).toFixed(2);
    const requestsPerSecond = (this.stats.total / duration).toFixed(2);
    
    console.log('\n\n🎯 STRESS TEST RESULTS');
    console.log('═'.repeat(50));
    console.log(`⏱️  Duration: ${duration.toFixed(2)} seconds`);
    console.log(`📊 Total Requests: ${this.stats.total}`);
    console.log(`✅ Successful: ${this.stats.success} (${successRate}%)`);
    console.log(`❌ Failed: ${this.stats.failed}`);
    console.log(`🚀 Rate: ${requestsPerSecond} requests/second`);
    console.log(`⚡ Throughput: ${(this.stats.success / duration).toFixed(2)} successful req/s`);
    
    if (this.stats.failed > 0) {
      console.log('\n💥 ERROR BREAKDOWN:');
      Object.entries(this.stats.errors).forEach(([error, count]) => {
        console.log(`   ${error}: ${count} times`);
      });
    }
    
    console.log('\n📈 PERFORMANCE RATING:');
    if (successRate >= 99) {
      console.log('🏆 EXCELLENT - 99%+ success rate');
    } else if (successRate >= 95) {
      console.log('🥇 GREAT - 95%+ success rate'); 
    } else if (successRate >= 90) {
      console.log('🥈 GOOD - 90%+ success rate');
    } else if (successRate >= 80) {
      console.log('🥉 FAIR - 80%+ success rate');
    } else {
      console.log('⚠️  POOR - <80% success rate');
    }
    
    console.log('\n🎉 Stress test completed!');
  }
}

async function main() {
  const stressTest = new StressTest();
  
  try {
    // Default to 30 seconds, or use command line argument
    const duration = process.argv[2] ? parseInt(process.argv[2]) : 30;
    await stressTest.runStressTest(duration);
  } catch (error) {
    console.error('💥 Stress test failed:', error.message);
    process.exit(1);
  }
}

main().catch(console.error);