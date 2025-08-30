import { useQuery, useMutation, UseQueryOptions, UseMutationOptions } from '@tanstack/react-query';
import axios from 'axios';
import * as Types from './types';

// Query Keys
export const orderKeys = {
  getOrder: () => ['order', 'getOrder'] as const,
  getUserOrders: () => ['order', 'getUserOrders'] as const,
  getOrdersByStatus: () => ['order', 'getOrdersByStatus'] as const,
} as const;

// Query Hooks
export function useGetOrder(
  
  options?: Omit<UseQueryOptions<GetOrderResponse>, 'queryKey' | 'queryFn'>
) {
  return useQuery({
    queryKey: orderKeys.getOrder(),
    queryFn: async () => {
      const response = await axios.get(`/api/v1/order-service/orders/{id}`);
      return response.data;
    },
    ...options,
  });
}

export function useGetUserOrders(
  
  options?: Omit<UseQueryOptions<GetUserOrdersResponse>, 'queryKey' | 'queryFn'>
) {
  return useQuery({
    queryKey: orderKeys.getUserOrders(),
    queryFn: async () => {
      const response = await axios.get(`/api/v1/order-service/orders/user/{user_id}`);
      return response.data;
    },
    ...options,
  });
}

export function useGetOrdersByStatus(
  
  options?: Omit<UseQueryOptions<GetOrdersByStatusResponse>, 'queryKey' | 'queryFn'>
) {
  return useQuery({
    queryKey: orderKeys.getOrdersByStatus(),
    queryFn: async () => {
      const response = await axios.get(`/api/v1/order-service/orders/status/{status}`);
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
  options?: UseMutationOptions<UpdateOrderResponse, Error, { data: any }>
) {
  return useMutation({
    mutationFn: async (variables: { data: any }) => {
      
      const response = await axios.put(`/api/v1/order-service/orders/{id}`, variables.data);
      return response.data;
    },
    ...options,
  });
}

export function useDeleteOrder(
  options?: UseMutationOptions<DeleteOrderResponse, Error, void>
) {
  return useMutation({
    mutationFn: async (variables: void) => {
      
      const response = await axios.delete(`/api/v1/order-service/orders/{id}`);
      return response.data;
    },
    ...options,
  });
}

export function useConfirmOrder(
  options?: UseMutationOptions<ConfirmOrderResponse, Error, { data: any }>
) {
  return useMutation({
    mutationFn: async (variables: { data: any }) => {
      
      const response = await axios.post(`/api/v1/order-service/orders/{id}/confirm`, variables.data);
      return response.data;
    },
    ...options,
  });
}

export function useCancelOrder(
  options?: UseMutationOptions<CancelOrderResponse, Error, { data: any }>
) {
  return useMutation({
    mutationFn: async (variables: { data: any }) => {
      
      const response = await axios.post(`/api/v1/order-service/orders/{id}/cancel`, variables.data);
      return response.data;
    },
    ...options,
  });
}