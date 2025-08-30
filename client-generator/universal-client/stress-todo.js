#!/usr/bin/env node

const { RabbitMeshClient } = require('./dist/index.js');

class TodoStressTest {
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
      errors: {},
      responseTimes: []
    };
  }

  async runStressTest(durationSeconds = 30) {
    console.log(`🚀 Starting ${durationSeconds}s TODO-ONLY stress test...`);
    console.log('📊 Testing ONLY todo.createTodo() endpoint');
    console.log('⚡ Sending maximum burst requests...\n');

    this.stats.startTime = Date.now();
    const endTime = this.stats.startTime + (durationSeconds * 1000);
    
    // Create massive batches of concurrent requests
    let requestId = 0;
    
    while (Date.now() < endTime) {
      // Much larger batch size since we're only testing one endpoint
      const batchSize = 50; // Send 50 concurrent todo requests
      const batch = [];
      
      for (let i = 0; i < batchSize && Date.now() < endTime; i++) {
        batch.push(this.createSingleTodo(++requestId));
      }
      
      // Wait for current batch to complete
      await Promise.allSettled(batch);
      
      // Progress update every 200 requests  
      if (requestId % 200 === 0) {
        const elapsed = (Date.now() - this.stats.startTime) / 1000;
        const rate = this.stats.total / elapsed;
        process.stdout.write(`\r📈 ${this.stats.total} todos, ${rate.toFixed(1)} req/s, ${this.stats.success} success, ${this.stats.failed} failed`);
      }
    }
    
    this.stats.endTime = Date.now();
    this.printResults();
  }

  async createSingleTodo(requestId) {
    const startTime = Date.now();
    
    try {
      this.stats.total++;
      
      // Only test todo creation - no auth needed
      await this.client.todo.createTodo({
        title: `Stress Todo #${requestId}`,
        description: `High-speed stress test todo created at ${new Date().toISOString()}`
      });
      
      this.stats.success++;
      
      // Track response time
      const responseTime = Date.now() - startTime;
      this.stats.responseTimes.push(responseTime);
      
    } catch (error) {
      this.stats.failed++;
      
      // Track error types
      const errorType = error.response?.status || error.code || error.message || 'unknown';
      this.stats.errors[errorType] = (this.stats.errors[errorType] || 0) + 1;
    }
  }

  printResults() {
    const duration = (this.stats.endTime - this.stats.startTime) / 1000;
    const successRate = (this.stats.success / this.stats.total * 100).toFixed(2);
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
    
    console.log('\n\n🎯 TODO STRESS TEST RESULTS');
    console.log('═'.repeat(60));
    console.log(`⏱️  Duration: ${duration.toFixed(2)} seconds`);
    console.log(`📊 Total Todo Requests: ${this.stats.total}`);
    console.log(`✅ Successful: ${this.stats.success} (${successRate}%)`);
    console.log(`❌ Failed: ${this.stats.failed}`);
    console.log(`🚀 Total Rate: ${requestsPerSecond} requests/second`);
    console.log(`⚡ Success Rate: ${successPerSecond} successful todos/second`);
    console.log(`⏱️  Avg Response Time: ${avgResponseTime}ms`);
    console.log(`📈 95th Percentile: ${p95ResponseTime}ms`);
    
    if (this.stats.failed > 0) {
      console.log('\n💥 ERROR BREAKDOWN:');
      Object.entries(this.stats.errors).forEach(([error, count]) => {
        console.log(`   ${error}: ${count} times`);
      });
    }
    
    console.log('\n📈 PERFORMANCE RATING:');
    const rps = parseFloat(successPerSecond);
    if (rps >= 100) {
      console.log('🏆 BLAZING FAST - 100+ todos/second');
    } else if (rps >= 50) {
      console.log('🔥 VERY FAST - 50+ todos/second'); 
    } else if (rps >= 20) {
      console.log('🚀 FAST - 20+ todos/second');
    } else if (rps >= 10) {
      console.log('⚡ DECENT - 10+ todos/second');
    } else {
      console.log('🐌 SLOW - <10 todos/second');
    }
    
    console.log('\n🎉 Todo stress test completed!');
    console.log(`💪 Your RabbitMesh can handle ${Math.floor(rps)} todos per second!`);
  }
}

async function main() {
  const stressTest = new TodoStressTest();
  
  try {
    const duration = process.argv[2] ? parseInt(process.argv[2]) : 30;
    await stressTest.runStressTest(duration);
  } catch (error) {
    console.error('💥 Todo stress test failed:', error.message);
    process.exit(1);
  }
}

main().catch(console.error);