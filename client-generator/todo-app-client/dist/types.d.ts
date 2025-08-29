export interface UserRole {
    Admin: "admin";
    Editor: "editor";
    Author: "author";
    Reader: "reader";
}
export interface UserInfo {
    id: string;
    username: string;
    email: string;
    full_name?: string | null;
    avatar_url?: string | null;
    role: UserRole;
    created_at: string;
}
export interface RegisterRequest {
    username: string;
    email: string;
    password: string;
    full_name?: string;
}
export interface LoginRequest {
    email: string;
    password: string;
}
export interface AuthResponse {
    success: boolean;
    message: string;
    user?: UserInfo | null;
    token?: string | null;
}
export interface ValidateTokenRequest {
    token: string;
}
export interface ValidateTokenResponse {
    valid: boolean;
    user_id?: string | null;
    role?: UserRole | null;
    username?: string | null;
}
export interface UpdateProfileRequest {
    full_name?: string;
    avatar_url?: string;
}
export interface PostStatus {
    Draft: "draft";
    Published: "published";
    Archived: "archived";
}
export interface BlogPost {
    id: string;
    title: string;
    content: string;
    excerpt: string;
    author_id: string;
    author_name: string;
    slug: string;
    tags: string[];
    status: PostStatus;
    view_count: number;
    created_at: string;
    updated_at: string;
    published_at?: string | null;
}
export interface CreatePostRequest {
    title: string;
    content: string;
    tags: string[];
    status: PostStatus;
}
export interface UpdatePostRequest {
    title?: string;
    content?: string;
    tags?: string[];
    status?: PostStatus;
}
export interface PostResponse {
    success: boolean;
    message: string;
    post?: BlogPost | null;
}
export interface PostListResponse {
    success: boolean;
    message: string;
    posts: BlogPost[];
    total: number;
    page: number;
    per_page: number;
}
export interface Comment {
    id: string;
    post_id: string;
    author_id: string;
    author_name: string;
    content: string;
    parent_id?: string | null;
    created_at: string;
    updated_at: string;
}
export interface CreateCommentRequest {
    post_id: string;
    content: string;
    parent_id?: string | null;
}
export interface CommentResponse {
    success: boolean;
    message: string;
    comment?: Comment | null;
}
export interface CommentListResponse {
    success: boolean;
    message: string;
    comments: Comment[];
    total: number;
}
//# sourceMappingURL=types.d.ts.map