import { ServiceDefinition, GeneratorConfig } from './types';
export declare class ClientGenerator {
    private config;
    constructor(config: GeneratorConfig);
    generateClient(services: ServiceDefinition[]): Promise<void>;
    private generateTypes;
    private extractTypes;
    private addBlogPlatformTypes;
    private addTodoTypes;
    private generateTypesContent;
    private generateServiceClients;
    private generateServiceClient;
    private generateMethod;
    private generateMethodParams;
    private generateMainClient;
    private generatePackageJson;
    private generateTsConfig;
    private generateIndex;
    private capitalize;
}
//# sourceMappingURL=generator.d.ts.map