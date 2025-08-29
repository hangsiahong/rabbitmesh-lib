import { AxiosInstance } from 'axios';
export declare class NotificationClient {
    private http;
    constructor(http: AxiosInstance);
    sendNotification(data: any): Promise<any>;
    getNotificationHistory(): Promise<any>;
    streamNotifications(): Promise<any>;
    getNotificationStats(): Promise<any>;
}
