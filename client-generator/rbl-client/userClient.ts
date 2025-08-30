import { useQuery, useMutation, UseQueryOptions, UseMutationOptions } from '@tanstack/react-query';
import axios from 'axios';
import * as Types from './types';

// Query Keys
export const userKeys = {
  getUser: (id: string) => ['user', 'getUser', id] as const,
  listUsers: () => ['user', 'listUsers'] as const,
  getUserByEmail: (email: string) => ['user', 'getUserByEmail', email] as const,
} as const;

// Query Hooks
export function useGetUser(
  id: string,
  
  options?: Omit<UseQueryOptions<GetUserResponse>, 'queryKey' | 'queryFn'>
) {
  return useQuery({
    queryKey: userKeys.getUser(id),
    queryFn: async () => {
      const response = await axios.get(`/api/v1/user-service/users/${id}`);
      return response.data;
    },
    ...options,
  });
}

export function useListUsers(
  
  options?: Omit<UseQueryOptions<ListUsersResponse>, 'queryKey' | 'queryFn'>
) {
  return useQuery({
    queryKey: userKeys.listUsers(),
    queryFn: async () => {
      const response = await axios.get(`/api/v1/user-service/users`);
      return response.data;
    },
    ...options,
  });
}

export function useGetUserByEmail(
  email: string,
  
  options?: Omit<UseQueryOptions<GetUserByEmailResponse>, 'queryKey' | 'queryFn'>
) {
  return useQuery({
    queryKey: userKeys.getUserByEmail(email),
    queryFn: async () => {
      const response = await axios.get(`/api/v1/user-service/users/email/${email}`);
      return response.data;
    },
    ...options,
  });
}

// Mutation Hooks
export function useCreateUser(
  options?: UseMutationOptions<CreateUserResponse, Error, { data: any }>
) {
  return useMutation({
    mutationFn: async (variables: { data: any }) => {
      
      const response = await axios.post(`/api/v1/user-service/users`, variables.data);
      return response.data;
    },
    ...options,
  });
}

export function useUpdateUser(
  options?: UseMutationOptions<UpdateUserResponse, Error, { id: string; data: any }>
) {
  return useMutation({
    mutationFn: async (variables: { id: string; data: any }) => {
      const id = variables.id;
      const response = await axios.put(`/api/v1/user-service/users/${id}`, variables.data);
      return response.data;
    },
    ...options,
  });
}

export function useDeleteUser(
  options?: UseMutationOptions<DeleteUserResponse, Error, { id: string }>
) {
  return useMutation({
    mutationFn: async (variables: { id: string }) => {
      const id = variables.id;
      const response = await axios.delete(`/api/v1/user-service/users/${id}`);
      return response.data;
    },
    ...options,
  });
}