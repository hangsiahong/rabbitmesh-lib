import axios, { AxiosInstance } from 'axios';
import * as Types from './types';

export class NotificationClient {
  constructor(private http: AxiosInstance) {}

  async sendNotification(data: any): Promise<any> {
    const response = await this.http.post(`/api/v1/notification-service/send_notification`, data);
    return response.data;
  }

  async getNotificationHistory(): Promise<any> {
    const response = await this.http.get(`/api/v1/notification-service/get_notification_history`);
    return response.data;
  }

  async streamNotifications(): Promise<any> {
    const response = await this.http.get(`/api/v1/notification-service/stream_notifications`);
    return response.data;
  }

  async getNotificationStats(): Promise<any> {
    const response = await this.http.get(`/api/v1/notification-service/get_notification_stats`);
    return response.data;
  }
}