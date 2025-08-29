#!/usr/bin/env node

import { Command } from 'commander';
import { ServiceParser } from './parser';
import { ClientGenerator } from './generator';
import { GeneratorConfig } from './types';

const program = new Command();

program
  .name('rabbitmesh-client-generator')
  .description('Generate TypeScript client for RabbitMesh microservices')
  .version('1.0.0');

program
  .option('-u, --gateway-url <url>', 'Gateway URL', 'http://localhost:3000')
  .option('-o, --output <dir>', 'Output directory', './generated-client')
  .option('-p, --package-name <name>', 'Package name', '@your-org/rabbitmesh-client')
  .option('--include-services <services>', 'Comma-separated list of services to include')
  .option('--exclude-services <services>', 'Comma-separated list of services to exclude')
  .option('--manual', 'Use manual service definitions instead of fetching from gateway')
  .option('--blog-platform', 'Use blog platform service definitions (auth + blog services)')
  .action(async (options) => {
    try {
      console.log('🚀 Starting RabbitMesh client generation...');
      
      const config: GeneratorConfig = {
        gatewayUrl: options.gatewayUrl,
        outputDir: options.output,
        packageName: options.packageName,
        includeServices: options.includeServices?.split(','),
        excludeServices: options.excludeServices?.split(',')
      };

      const parser = new ServiceParser(config.gatewayUrl);
      const generator = new ClientGenerator(config);

      let services;
      if (options.blogPlatform) {
        console.log('📝 Using blog platform service definitions...');
        services = parser.getManualBlogPlatformServices();
      } else if (options.manual) {
        console.log('📝 Using manual service definitions...');
        services = [parser.getManualTodoService()];
      } else {
        console.log('🔍 Discovering services from gateway...');
        try {
          services = await parser.parseServicesFromGateway();
        } catch (error) {
          console.log('⚠️  Failed to fetch from gateway, using manual definitions...');
          services = [parser.getManualTodoService()];
        }
      }

      // Filter services if specified
      if (config.includeServices) {
        services = services.filter(s => config.includeServices!.includes(s.name));
      }
      if (config.excludeServices) {
        services = services.filter(s => !config.excludeServices!.includes(s.name));
      }

      console.log(`📋 Found ${services.length} service(s): ${services.map(s => s.name).join(', ')}`);
      
      console.log('⚙️  Generating TypeScript client...');
      await generator.generateClient(services);
      
      console.log(`✅ Client generated successfully in: ${config.outputDir}`);
      console.log(`📦 Package name: ${config.packageName}`);
      console.log('');
      console.log('Next steps:');
      console.log(`  cd ${config.outputDir}`);
      console.log('  npm install');
      console.log('  npm run build');
      
    } catch (error) {
      console.error('❌ Generation failed:', error);
      process.exit(1);
    }
  });

program.parse();