#!/usr/bin/env node
"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const commander_1 = require("commander");
const parser_1 = require("./parser");
const generator_1 = require("./generator");
const program = new commander_1.Command();
program
    .name('rabbitmesh-client-generator')
    .description('Generate TypeScript client for RabbitMesh microservices')
    .version('1.0.0');
program
    .option('-u, --gateway-url <url>', 'Gateway URL (default: http://localhost:8081)', 'http://localhost:8081')
    .option('-o, --output <dir>', 'Output directory', './generated-client')
    .option('-p, --package-name <name>', 'Package name', '@your-org/rabbitmesh-client')
    .option('--include-services <services>', 'Comma-separated list of services to include')
    .option('--exclude-services <services>', 'Comma-separated list of services to exclude')
    .action(async (options) => {
    try {
        console.log('🚀 Starting RabbitMesh client generation...');
        const config = {
            gatewayUrl: options.gatewayUrl,
            outputDir: options.output,
            packageName: options.packageName,
            includeServices: options.includeServices?.split(','),
            excludeServices: options.excludeServices?.split(',')
        };
        const parser = new parser_1.ServiceParser(config.gatewayUrl);
        const generator = new generator_1.ClientGenerator(config);
        console.log('🔍 Discovering services from dynamic gateway...');
        console.log(`   Gateway URL: ${config.gatewayUrl}`);
        let services;
        try {
            services = await parser.parseServicesFromGateway();
            console.log(`✅ Successfully discovered ${services.length} services`);
        }
        catch (error) {
            console.error('❌ Failed to fetch services from gateway:', error);
            console.log('💡 Make sure your dynamic gateway is running and accessible');
            console.log(`   Try: curl ${config.gatewayUrl}/api/services`);
            process.exit(1);
        }
        // Filter services if specified
        if (config.includeServices) {
            services = services.filter(s => config.includeServices.includes(s.name));
        }
        if (config.excludeServices) {
            services = services.filter(s => !config.excludeServices.includes(s.name));
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
    }
    catch (error) {
        console.error('❌ Generation failed:', error);
        process.exit(1);
    }
});
program.parse();
//# sourceMappingURL=generate.js.map