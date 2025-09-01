import { useQuery, useMutation, UseQueryOptions, UseMutationOptions } from '@tanstack/react-query';
import axios from 'axios';
import * as Types from './types';

// Query Keys
export const authKeys = {
  getCurrentUser: () => ['auth', 'getCurrentUser'] as const,
} as const;

// Query Hooks
export function useGetCurrentUser(
  
  options?: Omit<UseQueryOptions<Types.GetCurrentUserResponse>, 'queryKey' | 'queryFn'>
) {
  return useQuery({
    queryKey: authKeys.getCurrentUser(),
    queryFn: async () => {
      const response = await axios.get(`/api/v1/auth-service/auth/me`);
      return response.data;
    },
    ...options,
  });
}

// Mutation Hooks
export function useLogin(
  options?: UseMutationOptions<Types.LoginResponse, Error, { data: any }>
) {
  return useMutation({
    mutationFn: async (variables: { data: any }) => {
      
      const response = await axios.post(`/api/v1/auth-service/auth/login`, variables.data);
      return response.data;
    },
    ...options,
  });
}

export function useValidateToken(
  options?: UseMutationOptions<Types.ValidateTokenResponse, Error, { data: any }>
) {
  return useMutation({
    mutationFn: async (variables: { data: any }) => {
      
      const response = await axios.post(`/api/v1/auth-service/auth/validate`, variables.data);
      return response.data;
    },
    ...options,
  });
}

export function useRefreshToken(
  options?: UseMutationOptions<Types.RefreshTokenResponse, Error, { data: any }>
) {
  return useMutation({
    mutationFn: async (variables: { data: any }) => {
      
      const response = await axios.post(`/api/v1/auth-service/auth/refresh`, variables.data);
      return response.data;
    },
    ...options,
  });
}

export function useCheckPermission(
  options?: UseMutationOptions<Types.CheckPermissionResponse, Error, { data: any }>
) {
  return useMutation({
    mutationFn: async (variables: { data: any }) => {
      
      const response = await axios.post(`/api/v1/auth-service/auth/check-permission`, variables.data);
      return response.data;
    },
    ...options,
  });
}

export function useLogout(
  options?: UseMutationOptions<Types.LogoutResponse, Error, { data: any }>
) {
  return useMutation({
    mutationFn: async (variables: { data: any }) => {
      
      const response = await axios.post(`/api/v1/auth-service/auth/logout`, variables.data);
      return response.data;
    },
    ...options,
  });
}