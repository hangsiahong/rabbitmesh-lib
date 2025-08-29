import { AxiosInstance } from 'axios';
import { TodoClient } from './todoClient';
export interface RabbitMeshClientConfig {
    baseURL: string;
    timeout?: number;
    headers?: Record<string, string>;
}
export declare class RabbitMeshClient {
    private http;
    readonly todo: TodoClient;
    constructor(config: RabbitMeshClientConfig | string);
    getHttpClient(): AxiosInstance;
}
//# sourceMappingURL=client.d.ts.map