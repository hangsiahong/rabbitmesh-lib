import { useQuery, useMutation, UseQueryOptions, UseMutationOptions } from '@tanstack/react-query';
import axios from 'axios';
import * as Types from './types';

// Query Keys
export const orderKeys = {
  getOrder: (id: string) => ['order', 'getOrder', id] as const,
  getUserOrders: (user_id: string) => ['order', 'getUserOrders', user_id] as const,
  getOrdersByStatus: (status: string) => ['order', 'getOrdersByStatus', status] as const,
} as const;

// Query Hooks
export function useGetOrder(
  id: string,
  
  options?: Omit<UseQueryOptions<GetOrderResponse>, 'queryKey' | 'queryFn'>
) {
  return useQuery({
    queryKey: orderKeys.getOrder(id),
    queryFn: async () => {
      const response = await axios.get(`/api/v1/order-service/orders/${id}`);
      return response.data;
    },
    ...options,
  });
}

export function useGetUserOrders(
  user_id: string,
  
  options?: Omit<UseQueryOptions<GetUserOrdersResponse>, 'queryKey' | 'queryFn'>
) {
  return useQuery({
    queryKey: orderKeys.getUserOrders(user_id),
    queryFn: async () => {
      const response = await axios.get(`/api/v1/order-service/orders/user/${user_id}`);
      return response.data;
    },
    ...options,
  });
}

export function useGetOrdersByStatus(
  status: string,
  
  options?: Omit<UseQueryOptions<GetOrdersByStatusResponse>, 'queryKey' | 'queryFn'>
) {
  return useQuery({
    queryKey: orderKeys.getOrdersByStatus(status),
    queryFn: async () => {
      const response = await axios.get(`/api/v1/order-service/orders/status/${status}`);
      return response.data;
    },
    ...options,
  });
}

// Mutation Hooks
export function useCreateOrder(
  options?: UseMutationOptions<CreateOrderResponse, Error, { data: any }>
) {
  return useMutation({
    mutationFn: async (variables: { data: any }) => {
      
      const response = await axios.post(`/api/v1/order-service/orders`, variables.data);
      return response.data;
    },
    ...options,
  });
}

export function useUpdateOrder(
  options?: UseMutationOptions<UpdateOrderResponse, Error, { id: string; data: any }>
) {
  return useMutation({
    mutationFn: async (variables: { id: string; data: any }) => {
      const id = variables.id;
      const response = await axios.put(`/api/v1/order-service/orders/${id}`, variables.data);
      return response.data;
    },
    ...options,
  });
}

export function useDeleteOrder(
  options?: UseMutationOptions<DeleteOrderResponse, Error, { id: string }>
) {
  return useMutation({
    mutationFn: async (variables: { id: string }) => {
      const id = variables.id;
      const response = await axios.delete(`/api/v1/order-service/orders/${id}`);
      return response.data;
    },
    ...options,
  });
}

export function useConfirmOrder(
  options?: UseMutationOptions<ConfirmOrderResponse, Error, { id: string; data: any }>
) {
  return useMutation({
    mutationFn: async (variables: { id: string; data: any }) => {
      const id = variables.id;
      const response = await axios.post(`/api/v1/order-service/orders/${id}/confirm`, variables.data);
      return response.data;
    },
    ...options,
  });
}

export function useCancelOrder(
  options?: UseMutationOptions<CancelOrderResponse, Error, { id: string; data: any }>
) {
  return useMutation({
    mutationFn: async (variables: { id: string; data: any }) => {
      const id = variables.id;
      const response = await axios.post(`/api/v1/order-service/orders/${id}/cancel`, variables.data);
      return response.data;
    },
    ...options,
  });
}