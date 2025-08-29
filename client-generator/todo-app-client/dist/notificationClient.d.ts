import { AxiosInstance } from 'axios';
export declare class NotificationClient {
    private http;
    constructor(http: AxiosInstance);
    sendNotification(user_id: string, type: string, subject: string, body: string): Promise<any>;
    getNotificationHistory(user_id: string): Promise<any>;
    streamNotifications(user_id: string): Promise<any>;
    getNotificationStats(): Promise<any>;
}
//# sourceMappingURL=notificationClient.d.ts.map