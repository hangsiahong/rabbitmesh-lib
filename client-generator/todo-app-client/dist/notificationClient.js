"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.NotificationClient = void 0;
class NotificationClient {
    constructor(http) {
        this.http = http;
    }
    async sendNotification(user_id, type, subject, body) {
        const response = await this.http.post(`/api/v1/notification-service/send_notification`, { user_id, type, subject, body });
        return response.data;
    }
    async getNotificationHistory(user_id) {
        const response = await this.http.get(`/api/v1/notification-service/get_notification_history`, { params: { user_id } });
        return response.data;
    }
    async streamNotifications(user_id) {
        const response = await this.http.get(`/api/v1/notification-service/stream_notifications`, { params: { user_id } });
        return response.data;
    }
    async getNotificationStats() {
        const response = await this.http.get(`/api/v1/notification-service/get_notification_stats`);
        return response.data;
    }
}
exports.NotificationClient = NotificationClient;
