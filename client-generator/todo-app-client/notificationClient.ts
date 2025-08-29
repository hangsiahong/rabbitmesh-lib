import axios, { AxiosInstance } from 'axios';
import * as Types from './types';

export class NotificationClient {
  constructor(private http: AxiosInstance) {}

  async sendNotification(user_id: string, type: string, subject: string, body: string): Promise<any> {
    const response = await this.http.post(`/api/v1/notification-service/send_notification`, { user_id, type, subject, body });
    return response.data;
  }

  async getNotificationHistory(user_id: string): Promise<any> {
    const response = await this.http.get(`/api/v1/notification-service/get_notification_history`, { params: { user_id } });
    return response.data;
  }

  async streamNotifications(user_id: string): Promise<any> {
    const response = await this.http.get(`/api/v1/notification-service/stream_notifications`, { params: { user_id } });
    return response.data;
  }

  async getNotificationStats(): Promise<any> {
    const response = await this.http.get(`/api/v1/notification-service/get_notification_stats`);
    return response.data;
  }
}