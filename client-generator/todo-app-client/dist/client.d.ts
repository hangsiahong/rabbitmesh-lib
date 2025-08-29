import { AxiosInstance } from 'axios';
import { AuthClient } from './authClient';
import { TodoClient } from './todoClient';
import { NotificationClient } from './notificationClient';
export interface RabbitMeshClientConfig {
    baseURL: string;
    timeout?: number;
    headers?: Record<string, string>;
}
export declare class RabbitMeshClient {
    private http;
    readonly auth: AuthClient;
    readonly todo: TodoClient;
    readonly notification: NotificationClient;
    constructor(config: RabbitMeshClientConfig | string);
    getHttpClient(): AxiosInstance;
}
//# sourceMappingURL=client.d.ts.map