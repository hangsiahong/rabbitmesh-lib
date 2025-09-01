import React, { useState, useEffect } from 'react'
import {
  useLogin,
  useGetCurrentUser,
  useListUsers,
  useCreateUser,
  useCreateOrder,
  useGetUserOrders,
  auth
} from '@hangsiahong/ecommerse-client'
import './App.css'

function App() {
  const [loginEmail, setLoginEmail] = useState('test@example.com')
  const [loginPassword, setLoginPassword] = useState('testpassword')
  const [userId, setUserId] = useState('test-user-123')
  const [results, setResults] = useState<string[]>([])
  const [authStatus, setAuthStatus] = useState({
    isAuthenticated: false,
    user: null as any,
    role: null as string | null,
    permissions: [] as string[]
  })

  const addResult = (result: string) => {
    setResults(prev => [result, ...prev].slice(0, 20)) // Keep last 20 results
  }

  // Monitor authentication status
  useEffect(() => {
    const updateAuthStatus = () => {
      setAuthStatus({
        isAuthenticated: auth.isAuthenticated(),
        user: auth.getCurrentUser(),
        role: auth.getRole(),
        permissions: auth.getPermissions()
      })
    }

    // Update immediately
    updateAuthStatus()

    // Update every second to show real-time auth status
    const interval = setInterval(updateAuthStatus, 1000)
    return () => clearInterval(interval)
  }, [])

  // Mutation hooks
  const loginMutation = useLogin({
    onSuccess: (data) => {
      addResult(`✅ Login Success: ${JSON.stringify(data?.data || data)}`)
      // Auth is now automatically managed - tokens are stored and injected!
      setTimeout(() => {
        addResult(`🔐 Auth Status: Authenticated=${auth.isAuthenticated()}, Role=${auth.getRole()}, Permissions=${auth.getPermissions().join(', ')}`)
      }, 100) // Small delay to let auth manager process the response
    },
    onError: (error: any) => {
      addResult(`⚠️ Login Error: ${error.response?.status} - ${error.response?.data?.error || error.message}`)
    }
  })

  const createUserMutation = useCreateUser({
    onSuccess: (data) => {
      addResult(`✅ Create User Success: ${JSON.stringify(data?.data || data)}`)
    },
    onError: (error: any) => {
      addResult(`⚠️ Create User Error: ${error.response?.status} - ${error.response?.data?.error || error.message}`)
    }
  })

  const createOrderMutation = useCreateOrder({
    onSuccess: (data) => {
      addResult(`✅ Create Order Success: ${JSON.stringify(data?.data || data)}`)
    },
    onError: (error: any) => {
      addResult(`⚠️ Create Order Error: ${error.response?.status} - ${error.response?.data?.error || error.message}`)
    }
  })

  // Query hooks
  const { data: currentUser, error: currentUserError, isLoading: currentUserLoading } = useGetCurrentUser()
  const { data: users, error: usersError, isLoading: usersLoading } = useListUsers()
  const { data: userOrders, error: userOrdersError, isLoading: userOrdersLoading } = useGetUserOrders(userId)

  // Handle query results
  React.useEffect(() => {
    if (currentUserError) {
      addResult(`⚠️ Get Current User Error: ${(currentUserError as any).response?.status} (Expected - no auth)`)
    } else if (currentUser) {
      addResult(`✅ Get Current User Success: ${JSON.stringify(currentUser)}`)
    }
  }, [currentUser, currentUserError])

  React.useEffect(() => {
    if (usersError) {
      addResult(`⚠️ List Users Error: ${(usersError as any).response?.status}`)
    } else if (users) {
      addResult(`✅ List Users Success: ${Array.isArray(users) ? `${users.length} users` : 'data received'}`)
    }
  }, [users, usersError])

  React.useEffect(() => {
    if (userOrdersError) {
      addResult(`⚠️ Get User Orders Error: ${(userOrdersError as any).response?.status}`)
    } else if (userOrders) {
      addResult(`✅ Get User Orders Success: ${JSON.stringify(userOrders)}`)
    }
  }, [userOrders, userOrdersError])

  const handleLogin = () => {
    addResult('🧪 Testing useLogin hook...')
    loginMutation.mutate({
      data: {
        email: loginEmail,
        password: loginPassword
      }
    })
  }

  const handleCreateUser = () => {
    addResult('🧪 Testing useCreateUser hook...')
    createUserMutation.mutate({
      data: {
        name: 'Test User from Vite',
        email: `vite-test-${Date.now()}@example.com`,
        password: 'vitetest123'
      }
    })
  }

  const handleCreateOrder = () => {
    addResult('🧪 Testing useCreateOrder hook...')
    createOrderMutation.mutate({
      data: {
        user_id: userId,
        items: [
          {
            product_id: 'vite-product-1',
            product_name: 'Vite Test Product 1',
            quantity: 2,
            unit_price: 19.99
          },
          {
            product_id: 'vite-product-2',
            product_name: 'Vite Test Product 2',
            quantity: 1,
            unit_price: 9.99
          }
        ],
        total_amount: 49.97
      }
    })
  }

  const handleLogout = () => {
    addResult('🚪 Logging out...')
    auth.logout()
    setTimeout(() => {
      addResult(`🔐 Auth Status after logout: Authenticated=${auth.isAuthenticated()}`)
    }, 100)
  }

  return (
    <div className="App" style={{ padding: '20px', fontFamily: 'monospace' }}>
      <h1>🧪 RabbitMesh Client - Live React Demo</h1>
      <p>Testing generated hooks: <code>@hangsiahong/ecommerse-client</code></p>
      <p>Gateway: <code>http://localhost:3333</code></p>
      
      {/* Authentication Status Display */}
      <div style={{ 
        marginTop: '10px', 
        padding: '15px', 
        background: authStatus.isAuthenticated ? '#e8f5e8' : '#fff3cd', 
        borderRadius: '5px',
        border: authStatus.isAuthenticated ? '2px solid #28a745' : '2px solid #ffc107'
      }}>
        <h3>🔐 Authentication Status</h3>
        <div style={{ fontSize: '12px', fontFamily: 'monospace' }}>
          <div><strong>Authenticated:</strong> {authStatus.isAuthenticated ? '✅ YES' : '❌ NO'}</div>
          {authStatus.user && (
            <>
              <div><strong>User:</strong> {authStatus.user.email} ({authStatus.user.name})</div>
              <div><strong>Role:</strong> {authStatus.role}</div>
              <div><strong>Permissions:</strong> [{authStatus.permissions.join(', ')}]</div>
              <div><strong>JWT Token:</strong> {auth.getToken() ? '✅ Present' : '❌ Missing'}</div>
            </>
          )}
        </div>
      </div>

      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '20px', marginTop: '20px' }}>
        {/* Controls */}
        <div>
          <h2>🎮 Test Controls</h2>
          
          <div style={{ marginBottom: '15px' }}>
            <h3>Authentication</h3>
            <div style={{ marginBottom: '10px' }}>
              <input
                type="email"
                placeholder="Email"
                value={loginEmail}
                onChange={(e) => setLoginEmail(e.target.value)}
                style={{ marginRight: '5px', padding: '5px' }}
              />
              <input
                type="password"
                placeholder="Password"
                value={loginPassword}
                onChange={(e) => setLoginPassword(e.target.value)}
                style={{ padding: '5px' }}
              />
            </div>
            <button 
              onClick={handleLogin}
              disabled={loginMutation.isPending}
              style={{ padding: '10px', marginRight: '5px' }}
            >
              {loginMutation.isPending ? 'Logging in...' : 'Test useLogin'}
            </button>
            {authStatus.isAuthenticated && (
              <button 
                onClick={handleLogout}
                style={{ padding: '10px', marginRight: '5px', background: '#dc3545', color: 'white', border: 'none', borderRadius: '3px' }}
              >
                🚪 Logout
              </button>
            )}
          </div>

          <div style={{ marginBottom: '15px' }}>
            <h3>User Management</h3>
            <button 
              onClick={handleCreateUser}
              disabled={createUserMutation.isPending}
              style={{ padding: '10px', marginRight: '5px' }}
            >
              {createUserMutation.isPending ? 'Creating...' : 'Test useCreateUser'}
            </button>
          </div>

          <div style={{ marginBottom: '15px' }}>
            <h3>Order Management</h3>
            <div style={{ marginBottom: '10px' }}>
              <input
                type="text"
                placeholder="User ID"
                value={userId}
                onChange={(e) => setUserId(e.target.value)}
                style={{ padding: '5px', width: '200px' }}
              />
            </div>
            <button 
              onClick={handleCreateOrder}
              disabled={createOrderMutation.isPending}
              style={{ padding: '10px' }}
            >
              {createOrderMutation.isPending ? 'Creating...' : 'Test useCreateOrder'}
            </button>
          </div>

          <div>
            <h3>📊 Query States</h3>
            <div style={{ fontSize: '12px' }}>
              <div>Current User: {currentUserLoading ? '🔄 Loading...' : currentUser ? '✅ Data' : '❌ Error'}</div>
              <div>Users List: {usersLoading ? '🔄 Loading...' : users ? '✅ Data' : '❌ Error'}</div>
              <div>User Orders: {userOrdersLoading ? '🔄 Loading...' : userOrders ? '✅ Data' : '❌ Error'}</div>
            </div>
          </div>
        </div>

        {/* Results */}
        <div>
          <h2>📋 Live Test Results</h2>
          <div style={{
            background: '#f5f5f5',
            padding: '10px',
            height: '500px',
            overflow: 'auto',
            fontSize: '12px',
            border: '1px solid #ddd'
          }}>
            {results.length === 0 ? (
              <p>Query hooks are running automatically... Results will appear here.</p>
            ) : (
              results.map((result, index) => (
                <div key={index} style={{ 
                  marginBottom: '8px', 
                  padding: '5px',
                  background: result.includes('✅') ? '#e8f5e8' : result.includes('⚠️') ? '#fff3cd' : '#e3f2fd',
                  borderRadius: '3px'
                }}>
                  <strong>{new Date().toLocaleTimeString()}</strong> - {result}
                </div>
              ))
            )}
          </div>
        </div>
      </div>

      <div style={{ marginTop: '20px', padding: '15px', background: '#f0f8ff', borderRadius: '5px' }}>
        <h3>🎉 Generated Hooks Working Live with JWT/RBAC/ABAC!</h3>
        <p>This Vite React app is using your generated RabbitMesh client with <strong>automatic authentication</strong>:</p>
        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '20px', textAlign: 'left' }}>
          <div>
            <h4>🔧 Generated Hooks:</h4>
            <ul>
              <li>✅ <code>useLogin</code> - Authentication mutation</li>
              <li>✅ <code>useCreateUser</code> - User creation mutation</li>
              <li>✅ <code>useCreateOrder</code> - Order creation mutation</li>
              <li>✅ <code>useGetCurrentUser</code> - Current user query</li>
              <li>✅ <code>useListUsers</code> - Users list query</li>
              <li>✅ <code>useGetUserOrders</code> - User orders query</li>
            </ul>
          </div>
          <div>
            <h4>🔐 Authentication Features:</h4>
            <ul>
              <li>✅ <strong>JWT</strong> - Automatic token storage & injection</li>
              <li>✅ <strong>RBAC</strong> - Role-Based Access Control</li>
              <li>✅ <strong>ABAC</strong> - Attribute-Based Access Control</li>
              <li>✅ <strong>Auto-refresh</strong> - Tokens refresh before expiry</li>
              <li>✅ <strong>Error handling</strong> - 401/403 errors handled automatically</li>
              <li>✅ <strong>TypeScript</strong> - Fully typed authentication API</li>
            </ul>
          </div>
        </div>
        <p><strong>🎯 Result:</strong> Login once, all protected endpoints work automatically!</p>
      </div>
    </div>
  )
}

export default App
