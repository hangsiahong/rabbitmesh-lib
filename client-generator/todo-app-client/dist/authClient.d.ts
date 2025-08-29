import { AxiosInstance } from 'axios';
export declare class AuthClient {
    private http;
    constructor(http: AxiosInstance);
    register(username: string, email: string, password: string): Promise<any>;
    login(username: string, password: string): Promise<any>;
    getProfile(user_id: string): Promise<any>;
    listUsers(): Promise<any>;
}
//# sourceMappingURL=authClient.d.ts.map