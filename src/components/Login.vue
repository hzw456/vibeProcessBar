<script setup lang="ts">
import { ref } from 'vue'
import { useAuthStore } from '../stores/auth'
import { useProgressStore } from '../stores/progressStore'

const emit = defineEmits<{
  (e: 'switch-to-register'): void
}>()

const authStore = useAuthStore()
const progressStore = useProgressStore()

const email = ref('')
const password = ref('')
const loading = ref(false)
const error = ref('')

async function handleSubmit() {
  error.value = ''
  loading.value = true

  try {
    await authStore.login(email.value, password.value)
    progressStore.setApiKey(authStore.apiKey)
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Login failed'
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <form class="auth-form" @submit.prevent="handleSubmit">
    <h2>Login</h2>

    <label class="auth-field">
      <span>Email</span>
      <input v-model="email" type="email" autocomplete="email" required />
    </label>

    <label class="auth-field">
      <span>Password</span>
      <input
        v-model="password"
        type="password"
        autocomplete="current-password"
        required
      />
    </label>

    <p v-if="error" class="auth-error">{{ error }}</p>

    <button type="submit" :disabled="loading">
      {{ loading ? 'Logging in...' : 'Login' }}
    </button>
    
    <p class="auth-switch">
      Don't have an account? 
      <a href="#" @click.prevent="emit('switch-to-register')">Register</a>
    </p>
  </form>
</template>

<style scoped>
.auth-form {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  width: 100%;
  max-width: 24rem;
  padding: 1.5rem;
  border: 1px solid #d0d7de;
  border-radius: 0.75rem;
  background: #fff;
}

.auth-field {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.auth-field input {
  padding: 0.75rem;
  border: 1px solid #c2c8cf;
  border-radius: 0.5rem;
  font: inherit;
}

.auth-error {
  margin: 0;
  color: #b42318;
}

button {
  padding: 0.75rem 1rem;
  border: none;
  border-radius: 0.5rem;
  background: #0f766e;
  color: #fff;
  font: inherit;
  cursor: pointer;
}

button:disabled {
  opacity: 0.7;
  cursor: wait;
}

.auth-switch {
  text-align: center;
  font-size: 0.875rem;
}

.auth-switch a {
  color: #0f766e;
  text-decoration: none;
}

.auth-switch a:hover {
  text-decoration: underline;
}
</style>
