import axios, { AxiosInstance } from 'axios';
import * as Types from './types';

export class BlogClient {
  constructor(private http: AxiosInstance) {}

  async createPost(authorization: string, request: CreatePostRequest): Promise<PostResponse> {
    const response = await this.http.post(`/posts`, authorization);
    return response.data;
  }

  async getPost(id: string): Promise<PostResponse> {
    const response = await this.http.get(`/posts/${id}`);
    return response.data;
  }

  async listPosts(page?: number, per_page?: number, status?: string): Promise<PostListResponse> {
    const response = await this.http.get(`/posts`, { params: { page, per_page, status } });
    return response.data;
  }

  async updatePost(authorization: string, id: string, request: UpdatePostRequest): Promise<PostResponse> {
    const response = await this.http.put(`/posts/${id}`, authorization);
    return response.data;
  }

  async deletePost(authorization: string, id: string): Promise<PostResponse> {
    const response = await this.http.delete(`/posts/${id}`, authorization);
    return response.data;
  }

  async createComment(authorization: string, post_id: string, request: CreateCommentRequest): Promise<CommentResponse> {
    const response = await this.http.post(`/posts/${post_id}/comments`, authorization);
    return response.data;
  }

  async getComments(post_id: string): Promise<CommentListResponse> {
    const response = await this.http.get(`/posts/${post_id}/comments`);
    return response.data;
  }
}