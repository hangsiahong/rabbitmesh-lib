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
}
//# sourceMappingURL=parser.d.ts.map