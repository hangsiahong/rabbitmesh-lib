import { AxiosInstance } from 'axios';
export declare class AuthClient {
    private http;
    constructor(http: AxiosInstance);
    register(data: any): Promise<any>;
    login(data: any): Promise<any>;
    getProfile(): Promise<any>;
    listUsers(): Promise<any>;
}
