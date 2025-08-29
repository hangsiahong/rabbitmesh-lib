"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.NotificationClient = void 0;
class NotificationClient {
    constructor(http) {
        this.http = http;
    }
    async sendNotification(data) {
        const response = await this.http.post(`/api/v1/notification-service/send_notification`, data);
        return response.data;
    }
    async getNotificationHistory() {
        const response = await this.http.get(`/api/v1/notification-service/get_notification_history`);
        return response.data;
    }
    async streamNotifications() {
        const response = await this.http.get(`/api/v1/notification-service/stream_notifications`);
        return response.data;
    }
    async getNotificationStats() {
        const response = await this.http.get(`/api/v1/notification-service/get_notification_stats`);
        return response.data;
    }
}
exports.NotificationClient = NotificationClient;
