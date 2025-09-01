import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { configureRabbitMeshClient } from '@hangsiahong/ecommerse-client'
import './index.css'
import App from './App.tsx'

// Configure the RabbitMesh client with FULL AUTHENTICATION SUPPORT
configureRabbitMeshClient({
  baseURL: 'http://localhost:3333',
  auth: {
    autoManageTokens: true,    // ✅ Automatic JWT token storage and injection
    rbacEnabled: true,         // ✅ Role-Based Access Control
    abacEnabled: true,         // ✅ Attribute-Based Access Control  
    tokenStorage: 'localStorage', // Store tokens in localStorage
    refreshThreshold: 5,       // Refresh token 5 minutes before expiry
  }
})

console.log('🔐 RabbitMesh client configured with automatic JWT/RBAC/ABAC authentication!')

// Create a React Query client
const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: 1,
      staleTime: 5 * 60 * 1000, // 5 minutes
    },
    mutations: {
      retry: false,
    },
  },
})

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <QueryClientProvider client={queryClient}>
      <App />
    </QueryClientProvider>
  </StrictMode>,
)
