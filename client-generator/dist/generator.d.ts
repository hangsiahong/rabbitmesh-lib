import { ServiceDefinition, GeneratorConfig } from './types';
export declare class ClientGenerator {
    private config;
    constructor(config: GeneratorConfig);
    generateClient(services: ServiceDefinition[]): Promise<void>;
    private generateTypes;
    private extractTypes;
    private extractRequestType;
    private extractResponseType;
    private addGenericTypes;
    private getRequestTypeName;
    private getResponseTypeName;
    private mapParameterType;
    private generateTypesContent;
    private generateServiceClients;
    private generateServiceClient;
    private generateQueryHook;
    private generateMutationHook;
    private generateMutationBody;
    private getMutationVariablesType;
    private generateHookParams;
    private getQueryKeyParams;
    private getQueryKeyParamsUsage;
    private buildQueryConfig;
    private generateMethodParams;
    private generateMainClient;
    private generatePackageJson;
    private generateTsConfig;
    private generateIndex;
    private capitalize;
}
//# sourceMappingURL=generator.d.ts.map