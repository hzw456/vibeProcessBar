import { computed, ref } from 'vue'
import { defineStore } from 'pinia'

const API_BASE_URL = 'http://192.168.1.28:3010'
const AUTH_STORAGE_KEY = 'auth'

type AuthUser = Record<string, unknown> | null

interface AuthResponse {
  token?: string
  apiKey?: string
  user?: Record<string, unknown>
  message?: string
}

export const useAuthStore = defineStore('auth', () => {
  const token = ref('')
  const apiKey = ref('')
  const user = ref<AuthUser>(null)

  const isAuthenticated = computed(() => Boolean(token.value))

  function persist() {
    localStorage.setItem(
      AUTH_STORAGE_KEY,
      JSON.stringify({
        token: token.value,
        apiKey: apiKey.value,
        user: user.value,
      }),
    )
  }

  function setAuth(data: AuthResponse) {
    token.value = data.token ?? ''
    apiKey.value = data.apiKey ?? ''
    user.value = data.user ?? null
    persist()
  }

  async function requestAuth(path: string, email: string, password: string) {
    const response = await fetch(`${API_BASE_URL}${path}`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ email, password }),
    })

    const data = (await response.json().catch(() => ({}))) as AuthResponse

    if (!response.ok) {
      throw new Error(data.message || 'Authentication request failed')
    }

    setAuth(data)
    return data
  }

  async function login(email: string, password: string) {
    return requestAuth('/login', email, password)
  }

  async function register(email: string, password: string) {
    return requestAuth('/register', email, password)
  }

  function logout() {
    token.value = ''
    apiKey.value = ''
    user.value = null
    localStorage.removeItem(AUTH_STORAGE_KEY)
  }

  function init() {
    const raw = localStorage.getItem(AUTH_STORAGE_KEY)
    if (!raw) {
      return
    }

    try {
      const parsed = JSON.parse(raw) as AuthResponse
      token.value = parsed.token ?? ''
      apiKey.value = parsed.apiKey ?? ''
      user.value = parsed.user ?? null
    } catch {
      logout()
    }
  }

  return {
    token,
    apiKey,
    user,
    isAuthenticated,
    login,
    register,
    logout,
    init,
  }
})
