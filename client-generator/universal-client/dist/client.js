"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.RabbitMeshClient = void 0;
const axios_1 = __importDefault(require("axios"));
const authClient_1 = require("./authClient");
const todoClient_1 = require("./todoClient");
const notificationClient_1 = require("./notificationClient");
class RabbitMeshClient {
    constructor(config) {
        const clientConfig = typeof config === 'string'
            ? { baseURL: config }
            : config;
        this.http = axios_1.default.create({
            baseURL: clientConfig.baseURL,
            timeout: clientConfig.timeout || 10000,
            headers: {
                'Content-Type': 'application/json',
                ...clientConfig.headers
            }
        });
        this.auth = new authClient_1.AuthClient(this.http);
        this.todo = new todoClient_1.TodoClient(this.http);
        this.notification = new notificationClient_1.NotificationClient(this.http);
    }
    // Utility method to access underlying axios instance
    getHttpClient() {
        return this.http;
    }
}
exports.RabbitMeshClient = RabbitMeshClient;
