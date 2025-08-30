import { ServiceDefinition } from './types';
export declare class ServiceParser {
    private gatewayUrl;
    constructor(gatewayUrl: string);
    parseServicesFromGateway(): Promise<ServiceDefinition[]>;
    private parseServiceFromEndpoints;
    private parseEndpointToMethod;
    private inferParametersFromPath;
    private parseService;
    private parseMethod;
    private parseParameter;
    private parseHttpRoute;
    private mapRustTypeToTypeScript;
    private toCamelCase;
    getManualTodoAppServices(): ServiceDefinition[];
    getManualBlogPlatformServices(): ServiceDefinition[];
    getManualAuthService(): ServiceDefinition;
    getManualBlogService(): ServiceDefinition;
    getManualTodoService(): ServiceDefinition;
    getManualTodoAppService(): ServiceDefinition;
    getManualNotificationService(): ServiceDefinition;
}
//# sourceMappingURL=parser.d.ts.map