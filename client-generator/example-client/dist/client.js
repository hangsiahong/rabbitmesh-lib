"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.RabbitMeshClient = void 0;
const axios_1 = __importDefault(require("axios"));
const todoClient_1 = require("./todoClient");
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
        this.todo = new todoClient_1.TodoClient(this.http);
    }
    // Utility method to access underlying axios instance
    getHttpClient() {
        return this.http;
    }
}
exports.RabbitMeshClient = RabbitMeshClient;
//# sourceMappingURL=client.js.map